use opencv::{core, highgui, imgproc, prelude::*, video, videoio, Result};

pub fn main() -> Result<()> {
    /*  Open a video file */
    let video_file = "./video/sample1.mp4";
    let mut cap = videoio::VideoCapture::from_file(video_file, videoio::CAP_ANY)?;
    if !cap.is_opened()? {
        panic!("Failed to open video file!");
    }

    /*  Prepare storage for previous grayscale frame & optical flow data */
    let mut prev_gray = Mat::default();

    /* Variables for summing RBC velocities across frames */
    let mut velocity_sum: f64 = 0.0;
    let mut valid_frames: usize = 0;

    loop {
        let mut frame = Mat::default();
        cap.read(&mut frame)?;
        if frame.empty() {
            println!("End of video stream");
            break;
        }

        /*  Convert to grayscale (needed for optical flow) */
        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        /*  If we have a valid previous frame, compute optical flow */
        if !prev_gray.empty() {
            // Farneback optical flow result: 2-channel float32 matrix
            // (channel 0 = flow in X, channel 1 = flow in Y)
            let mut flow = Mat::default();

            // Parameters often need tuning:
            //   - pyramid_scale (0.5)
            //   - levels (3)
            //   - winsize (15)
            //   - iterations (3)
            //   - poly_n (5)
            //   - poly_sigma (1.2)
            //   - flags (0)
            video::calc_optical_flow_farneback(
                &prev_gray, &gray, &mut flow, 0.5, 3, 15, 3, 5, 1.2, 0,
            )?;

            /*  Calculate velocity magnitude (RBC velocity in px/frame) */
            // Flow is a 2-channel Mat: (dx, dy)
            // We'll create separate Mats for dx, dy, then compute magnitude
            let mut flow_xy = core::Vector::<Mat>::new();
            core::split(&flow, &mut flow_xy)?;
            let flow_xy_vec = flow_xy.to_vec();
            let dx = &flow_xy_vec[0];
            let dy = &flow_xy_vec[1];

            let mut mag = Mat::default();
            let mut ang = Mat::default();
            core::cart_to_polar(dx, dy, &mut mag, &mut ang, false)?;

            /* Optionally, average the magnitude over the whole frame or over an ROI if you have a vessel mask */
            let mean_scalar = core::mean(&mag, &Mat::default())?;
            let mean_rbc_velocity = mean_scalar[0];

            velocity_sum += mean_rbc_velocity;
            valid_frames += 1;
            println!(
                "Frame #{} RBC velocity (px/frame) = {:.2}",
                valid_frames, mean_rbc_velocity
            );

            /* Visualize flow for debugging: create HSV or colar-coded flow map*/
            // Skipped here for brevity. You might see tutorials on how to convert (mag, ang) to a color wheel.

            /* Show original video or any debug overlay */
            highgui::imshow("Video Stream", &frame)?;
        } else {
            highgui::imshow("Video Stream", &frame)?;
        }

        /* Update previous frame */
        prev_gray = gray.clone();

        /* Key event handling (Esc to quit) */
        let key = highgui::wait_key(30)?;
        if key == 27 {
            break;
        }
    }

    /* After the loop: compute average RBC velocity over all processed frames */
    if valid_frames > 0 {
        let average_rbc_velocity = velocity_sum / valid_frames as f64;
        println!("------------------------------------------");
        println!(
            "Average RBC velocity for entire video: {:.2} px/frame",
            average_rbc_velocity
        );
    } else {
        println!("No valid frames processed for optical flow.");
    }

    Ok(())
}
