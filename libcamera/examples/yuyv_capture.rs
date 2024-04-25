use std::time::Duration;

use drm_fourcc::DrmFourcc;
use image::{ExtendedColorType, ImageBuffer};
use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    controls::FrameDurationLimits,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    properties,
    stream::StreamRole,
};

const PIXEL_FORMAT_YUYV: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'Y', b'U', b'Y', b'V']), 0);
// RGB888

// const PIXEL_FORMAT_RGB888: PixelFormat =
//     PixelFormat::new(u32::from_le_bytes([b'X', b'R', b'G', b'B', b'8', b'8', b'8', b'8']), 0);
const PIXEL_FORMAT_RGB888: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'R', b'G', b'B', b'8']), 0);

fn main() {
    // let image_input_encoding = DrmFourcc::Xrgb8888;
    // let image_input_encoding = DrmFourcc::Rgb565;
    let main_start = std::time::Instant::now();
    let image_input_encoding = DrmFourcc::Nv21;
    let pixel_format = PixelFormat::new(image_input_encoding as u32, 0);
    let filename = std::env::args().nth(1).expect("Usage: ./still <output(.jpg)|(.png)>");
    let mgr = CameraManager::new().unwrap();
    let cameras = mgr.cameras();
    let cam = cameras.get(0).expect("No cameras found");

    println!(
        "Using camera: {}",
        *cam.properties().get::<properties::Model>().unwrap()
    );

    let mut cam = cam.acquire().expect("Unable to acquire camera");

    // This will generate a default configuration for each specified role.
    let mut cfgs = cam.generate_configuration(&[StreamRole::ViewFinder]).unwrap();

    // Use YUYV format for capturing.
    // cfgs.get_mut(0).unwrap().set_pixel_format(PixelFormat::YUYV);
    // cfgs.get_mut(0).unwrap().set_pixel_format(PIXEL_FORMAT_YUYV);
    cfgs.get_mut(0).unwrap().set_pixel_format(pixel_format);
    let image_size = libcamera::geometry::Size {
        width: 3200,
        height: 2400,
    };
    cfgs.get_mut(0).unwrap().set_size(image_size);

    let size = cfgs.get(0).unwrap().get_size();
    let width = size.width;
    let height = size.height;
    println!("Size: W: {}, H: {}", size.width, size.height);

    println!("Generated config: {:#?}", cfgs);
    match cfgs.validate() {
        CameraConfigurationStatus::Valid => println!("Camera configuration valid!"),
        CameraConfigurationStatus::Adjusted => println!("Camera configuration adjusted: {:#?}", cfgs),
        CameraConfigurationStatus::Invalid => panic!("Error validating camera configuration"),
    }

    assert_eq!(
        cfgs.get(0).unwrap().get_pixel_format(),
        // PIXEL_FORMAT_YUYV,
        // PIXEL_FORMAT_RGB888,
        pixel_format,
        "RGB888 is not supported by the camera"
    );

    cam.configure(&mut cfgs).expect("Unable to configure camera");

    let mut alloc = FrameBufferAllocator::new(&cam);

    // Allocate frame buffers for the stream
    let cfg = cfgs.get(0).unwrap();
    let stream = cfg.stream().unwrap();
    let buffers = alloc.alloc(&stream).unwrap();
    println!("Allocated {} buffers", buffers.len());

    // Convert FrameBuffer to MemoryMappedFrameBuffer, which allows reading &[u8]
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    let mut reqs = buffers
        .into_iter()
        .map(|buf| {
            let mut req = cam.create_request(None).unwrap();
            req.add_buffer(&stream, buf).unwrap();
            req
        })
        .collect::<Vec<_>>();

    // Completed capture requests are returned as a callback.
    let (tx, rx) = std::sync::mpsc::channel();
    cam.on_request_completed(move |req| {
        tx.send(req).unwrap();
    });

    cam.start(None).unwrap();

    // Queue the capture request.
    cam.queue_request(reqs.pop().unwrap()).unwrap();

    println!("Waiting for camera request execution");
    let req = rx.recv_timeout(Duration::from_secs(2)).expect("Camera request failed");

    println!("Camera request {:?} completed!", req);

    // Get the framebuffer for our stream
    let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = match req.buffer(&stream) {
        Some(buffer) => buffer as &MemoryMappedFrameBuffer<FrameBuffer>,
        None => {
            eprintln!("Failed to retrieve buffer from the completed request");
            return;
        }
    };
    println!("FrameBuffer metadata: {:#?}", framebuffer.metadata());

    // YUYV format has only one data plane so the first element is our image data
    let planes = framebuffer.data();
    let yuyv_data = planes[0];

    // // image::save_buffer("hello.png", &yuyv_data, width, height, ExtendedColorType::L8).unwrap();
    // let start = std::time::Instant::now();
    // image::save_buffer(filename, &yuyv_data, width, height, ExtendedColorType::L8).unwrap();
    // let end = start.elapsed();
    // println!("image::save_buffer took: {end:?}");

    let start = std::time::Instant::now();
    match ImageBuffer::<image::Luma<u8>, &[u8]>::from_raw(width, height, yuyv_data) {
        Some(image_buf) => {
            image_buf.save("img_buff.png").unwrap();
        }
        None => eprintln!("Error trying to create ImageBuffer from raw data."),
    }
    let end = start.elapsed();
    println!("from_raw took: {end:?}");
    let main_end = main_start.elapsed();
    println!("The whole main function took: {main_end:?}");
    // let image_buf = ImageBuffer::from_raw(width, height, &yuyv_data);
    // Actual YUYV-encoded data will be smalled than framebuffer size, its length can be obtained from metadata.
    // let yuyv_len = framebuffer.metadata().unwrap().planes().get(0).unwrap().bytes_used as usize;
    // let rgb_data = yuyv_to_rgb(&yuyv_data[..yuyv_len]);
    // println!("rgb_data_bytes: {}", rgb_data.len());
    // std::fs::write(&filename, &rgb_data).unwrap();
    // std::fs::write(&filename, &rgb_data[..yuyv_len]).unwrap();

    // let yuyv_data: Vec<u8> = planes.iter().cloned().flatten().copied().collect();
    // let rgb_data = yuyv_to_rgb(&yuyv_data);

    // Actual JPEG-encoded data will be smalled than framebuffer size, its length can be obtained from metadata.
    // let yuyv_len = framebuffer.metadata().unwrap().planes().get(0).unwrap().bytes_used as usize;
    // std::fs::write(&filename, &yuyv_data[..yuyv_len]).unwrap();
    // std::fs::write(&filename, rgb_data).unwrap();
    // println!("Written RGB data to {}", filename);

    // let mut pixel_data = Vec::new();
    // for y in yuyv_data {
    //     let rgb_bytes = yuyv_to_rgb(y);

    // }

    // std::fs::write(&filename, rgb_bytes).unwrap();
    // println!("Written RGB data to {}", filename);

    // Everything is cleaned up automatically by Drop implementations.
}
