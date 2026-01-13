//! WebUSB support for Brother QL printers
//!
//! Uses raw wasm-bindgen bindings since web-sys doesn't have stable WebUSB support.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Array, Reflect, Uint8Array, Promise, Object};

use crate::error::UsbError;

/// Brother USB Vendor ID
const BROTHER_VENDOR_ID: u16 = 0x04f9;

/// Known Brother QL printer product IDs
const PRODUCT_IDS: &[(u16, &str)] = &[
    (0x2015, "QL-500"),
    (0x2016, "QL-550"),
    (0x2027, "QL-560"),
    (0x2028, "QL-570"),
    (0x2029, "QL-580N"),
    (0x201b, "QL-650TD"),
    (0x2042, "QL-700"),
    (0x2043, "QL-710W"),
    (0x2044, "QL-720NW"),
    (0x209b, "QL-800"),
    (0x209c, "QL-810W"),
    (0x209d, "QL-820NWB"),
    (0x20af, "QL-600"),
];

// Bindings for WebUSB API
#[wasm_bindgen]
extern "C" {
    type USB;
    type USBDevice;
    type Navigator;

    #[wasm_bindgen(js_name = navigator)]
    static NAVIGATOR: Navigator;

    #[wasm_bindgen(method, getter)]
    fn usb(this: &Navigator) -> Option<USB>;

    #[wasm_bindgen(method, js_name = requestDevice)]
    fn request_device(this: &USB, options: &JsValue) -> Promise;

    #[wasm_bindgen(method, js_name = getDevices)]
    fn get_devices(this: &USB) -> Promise;

    #[wasm_bindgen(method)]
    fn open(this: &USBDevice) -> Promise;

    #[wasm_bindgen(method)]
    fn close(this: &USBDevice) -> Promise;

    #[wasm_bindgen(method, js_name = claimInterface)]
    fn claim_interface(this: &USBDevice, interface_number: u8) -> Promise;

    #[wasm_bindgen(method, js_name = releaseInterface)]
    fn release_interface(this: &USBDevice, interface_number: u8) -> Promise;

    #[wasm_bindgen(method, js_name = transferOut)]
    fn transfer_out(this: &USBDevice, endpoint: u8, data: &Uint8Array) -> Promise;

    #[wasm_bindgen(method, js_name = transferIn)]
    fn transfer_in(this: &USBDevice, endpoint: u8, length: u32) -> Promise;

    #[wasm_bindgen(method, getter, js_name = vendorId)]
    fn vendor_id(this: &USBDevice) -> u16;

    #[wasm_bindgen(method, getter, js_name = productId)]
    fn product_id(this: &USBDevice) -> u16;

    #[wasm_bindgen(method, getter, js_name = productName)]
    fn product_name(this: &USBDevice) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name = serialNumber)]
    fn serial_number(this: &USBDevice) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    fn configuration(this: &USBDevice) -> Option<JsValue>;
}

/// WebUSB connection to a Brother QL printer
#[wasm_bindgen]
pub struct BrotherQlPrinter {
    device: USBDevice,
    interface_number: u8,
    endpoint_out: u8,
    endpoint_in: u8,
}

#[wasm_bindgen]
impl BrotherQlPrinter {
    /// Check if WebUSB is supported in the current browser
    #[wasm_bindgen(js_name = isSupported)]
    pub fn is_supported() -> bool {
        NAVIGATOR.usb().is_some()
    }

    /// Request access to a Brother QL printer
    /// 
    /// This must be called from a user gesture (e.g., button click)
    #[wasm_bindgen(js_name = requestDevice)]
    pub async fn request_device() -> Result<BrotherQlPrinter, UsbError> {
        let usb = NAVIGATOR.usb()
            .ok_or_else(|| UsbError::new("WebUSB not supported"))?;

        // Create filter options
        let filters = Array::new();
        let filter = Object::new();
        Reflect::set(&filter, &"vendorId".into(), &JsValue::from(BROTHER_VENDOR_ID))
            .map_err(|_| UsbError::new("Failed to set vendorId"))?;
        filters.push(&filter);

        let options = Object::new();
        Reflect::set(&options, &"filters".into(), &filters)
            .map_err(|_| UsbError::new("Failed to set filters"))?;

        let device_promise = usb.request_device(&options.into());
        let device: USBDevice = JsFuture::from(device_promise)
            .await
            .map_err(|e| UsbError::from(e))?
            .unchecked_into();

        Self::from_device(device).await
    }

    /// Get a list of already-paired devices
    #[wasm_bindgen(js_name = getDevices)]
    pub async fn get_devices() -> Result<Vec<BrotherQlPrinter>, UsbError> {
        let usb = NAVIGATOR.usb()
            .ok_or_else(|| UsbError::new("WebUSB not supported"))?;

        let devices_promise = usb.get_devices();
        let devices_array: Array = JsFuture::from(devices_promise)
            .await
            .map_err(|e| UsbError::from(e))?
            .unchecked_into();

        let mut printers = Vec::new();
        for i in 0..devices_array.length() {
            let device: USBDevice = devices_array.get(i).unchecked_into();
            if device.vendor_id() == BROTHER_VENDOR_ID {
                if let Ok(printer) = Self::from_device(device).await {
                    printers.push(printer);
                }
            }
        }

        Ok(printers)
    }

    async fn from_device(device: USBDevice) -> Result<Self, UsbError> {
        // Open the device
        JsFuture::from(device.open())
            .await
            .map_err(|e| UsbError::from(e))?;

        // Find the printer interface and endpoints
        let (interface_number, endpoint_out, endpoint_in) = Self::find_printer_endpoints(&device)?;

        // Claim the interface
        JsFuture::from(device.claim_interface(interface_number))
            .await
            .map_err(|e| UsbError::from(e))?;

        Ok(Self {
            device,
            interface_number,
            endpoint_out,
            endpoint_in,
        })
    }

    fn find_printer_endpoints(device: &USBDevice) -> Result<(u8, u8, u8), UsbError> {
        let configuration = device.configuration()
            .ok_or_else(|| UsbError::new("No active configuration"))?;
        
        let interfaces = Reflect::get(&configuration, &"interfaces".into())
            .map_err(|_| UsbError::new("Failed to get interfaces"))?;
        let interfaces: Array = interfaces.unchecked_into();
        
        for i in 0..interfaces.length() {
            let interface = interfaces.get(i);
            let alternates = Reflect::get(&interface, &"alternates".into())
                .map_err(|_| UsbError::new("Failed to get alternates"))?;
            let alternates: Array = alternates.unchecked_into();
            
            for j in 0..alternates.length() {
                let alternate = alternates.get(j);
                
                // Check if this is a printer class interface (class 7)
                let interface_class = Reflect::get(&alternate, &"interfaceClass".into())
                    .map_err(|_| UsbError::new("Failed to get interface class"))?
                    .as_f64()
                    .unwrap_or(0.0) as u8;
                
                if interface_class == 7 {
                    // Printer class
                    let interface_number = Reflect::get(&interface, &"interfaceNumber".into())
                        .map_err(|_| UsbError::new("Failed to get interface number"))?
                        .as_f64()
                        .unwrap_or(0.0) as u8;
                    
                    let endpoints = Reflect::get(&alternate, &"endpoints".into())
                        .map_err(|_| UsbError::new("Failed to get endpoints"))?;
                    let endpoints: Array = endpoints.unchecked_into();
                    
                    let mut endpoint_out = None;
                    let mut endpoint_in = None;
                    
                    for k in 0..endpoints.length() {
                        let endpoint = endpoints.get(k);
                        let direction = Reflect::get(&endpoint, &"direction".into())
                            .map_err(|_| UsbError::new("Failed to get direction"))?
                            .as_string()
                            .unwrap_or_default();
                        let endpoint_number = Reflect::get(&endpoint, &"endpointNumber".into())
                            .map_err(|_| UsbError::new("Failed to get endpoint number"))?
                            .as_f64()
                            .unwrap_or(0.0) as u8;
                        
                        if direction == "out" {
                            endpoint_out = Some(endpoint_number);
                        } else if direction == "in" {
                            endpoint_in = Some(endpoint_number);
                        }
                    }
                    
                    if let (Some(out), Some(inp)) = (endpoint_out, endpoint_in) {
                        return Ok((interface_number, out, inp));
                    }
                }
            }
        }
        
        Err(UsbError::new("No printer interface found"))
    }

    /// Get the product name of the connected printer
    #[wasm_bindgen(getter, js_name = productName)]
    pub fn get_product_name(&self) -> Option<String> {
        self.device.product_name()
    }

    /// Get the serial number of the connected printer
    #[wasm_bindgen(getter, js_name = serialNumber)]
    pub fn get_serial_number(&self) -> Option<String> {
        self.device.serial_number()
    }

    /// Get the product ID
    #[wasm_bindgen(getter, js_name = productId)]
    pub fn get_product_id(&self) -> u16 {
        self.device.product_id()
    }

    /// Get the model name based on product ID
    #[wasm_bindgen(getter, js_name = modelName)]
    pub fn model_name(&self) -> String {
        let pid = self.device.product_id();
        PRODUCT_IDS
            .iter()
            .find(|(id, _)| *id == pid)
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| format!("Unknown (0x{:04x})", pid))
    }

    /// Send raw data to the printer
    #[wasm_bindgen(js_name = sendRawData)]
    pub async fn send_raw_data(&self, data: &[u8]) -> Result<(), UsbError> {
        let array = Uint8Array::from(data);
        
        JsFuture::from(self.device.transfer_out(self.endpoint_out, &array))
            .await
            .map_err(|e| UsbError::from(e))?;
        
        Ok(())
    }

    /// Print a compiled print job
    #[wasm_bindgen]
    pub async fn print(&self, job: &crate::PrintJob) -> Result<(), UsbError> {
        let data = job.compile();
        self.send_raw_data(&data).await
    }

    /// Read status from the printer (32 bytes)
    #[wasm_bindgen(js_name = readStatus)]
    pub async fn read_status(&self) -> Result<Vec<u8>, UsbError> {
        let result = JsFuture::from(self.device.transfer_in(self.endpoint_in, 32))
            .await
            .map_err(|e| UsbError::from(e))?;
        
        let data = Reflect::get(&result, &"data".into())
            .map_err(|_| UsbError::new("Failed to get data from transfer result"))?;
        
        let buffer = Reflect::get(&data, &"buffer".into())
            .map_err(|_| UsbError::new("Failed to get buffer"))?;
        
        let array = Uint8Array::new(&buffer);
        Ok(array.to_vec())
    }

    /// Close the connection to the printer
    #[wasm_bindgen]
    pub async fn close(&self) -> Result<(), UsbError> {
        JsFuture::from(self.device.release_interface(self.interface_number))
            .await
            .map_err(|e| UsbError::from(e))?;
        
        JsFuture::from(self.device.close())
            .await
            .map_err(|e| UsbError::from(e))?;
        
        Ok(())
    }
}

/// Get list of supported printer models
#[wasm_bindgen(js_name = getSupportedPrinters)]
pub fn get_supported_printers() -> Vec<String> {
    PRODUCT_IDS.iter().map(|(_, name)| name.to_string()).collect()
}
