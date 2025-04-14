# Microvascular RBC Velocity Analysis with Rust + OpenCV

This repository demonstrates a **proof-of-concept** pipeline for analyzing **microvascular RBC velocity** from **handheld microscope** videos in real time using **Rust** and **OpenCV**. The main goal is to provide a **relatively low-cost, accessible** alternative to high-end devices (e.g., LSCI) for estimating blood flow in *microcirculation research*, particularly in **brain or spinal** contexts.

---

## Table of Contents

1. [Overview](#overview)  
2. [Approach & Process](#approach--process)  
   - [1. Reading Video Frames](#1-reading-video-frames)  
   - [2. (Optional) Video Stabilization](#2-optional-video-stabilization)  
   - [3. Preprocessing Steps](#3-preprocessing-steps)  
     - [3.1 Temporal or Spatial Denoising](#31-temporal-or-spatial-denoising)  
     - [3.2 Vessel Masking & ROI Definition](#32-vessel-masking--roi-definition)  
     - [3.3 Scale/Depth Change Handling (Z-Axis)](#33-scaledepth-change-handling-z-axis)  
   - [4. RBC Velocity Estimation](#4-rbc-velocity-estimation)  
   - [5. Output & Visualization](#5-output--visualization)  
3. [Challenges & Considerations](#challenges--considerations)  
   - [1. Severe Handheld Microscopy Constraints](#1-severe-handheld-microscopy-constraints)  
   - [2. ECC vs. Feature-Based Stabilization](#2-ecc-vs-feature-based-stabilization)  
   - [3. Advanced 2D/3D Compensation](#3-advanced-2d3d-compensation)  
   - [4. Parameter Tuning & Performance](#4-parameter-tuning--performance)  
4. [How to Run](#how-to-run)  
   - [Dependencies](#dependencies)  
   - [Build & Run](#build--run)  
5. [References](#references)

---

## Overview

In microcirculation research, **blood flow velocity** (e.g., RBC velocity in small arterioles, venules, and capillaries) is a key indicator of tissue health and function. Traditional instruments—like **Laser Speckle Contrast Imaging (LSCI)**—are highly effective but extremely expensive. The **handheld microscope** approach offers portability and cost savings but introduces **high sensitivity to motion** (including out-of-plane shifts) and **narrow depth of field**.

**This project** leverages OpenCV’s:
- **Optical flow** for velocity estimation (inspired by the [MicroTools paper](https://www.nature.com/articles/s42003-019-0473-8)),  
- **Stabilization** (via ECC or feature-based transforms) to correct for user hand jitter,  
- **Advanced preprocessing** (temporal/spatial denoising, ROI segmentation, scale/depth change mitigation),  
- **Rust** for safety, performance, and portability.

---

## Approach & Process

### 1. Reading Video Frames
We open a video file (`.mp4` or other supported formats) using Rust OpenCV’s `VideoCapture`.  
Each frame is read in a loop until the end of the video or the user presses **ESC**.

### 2. Video Stabilization
**Handheld microscope video** often suffers from intense shaking and drift. We provide **two** main strategies:

1. **Intensity-Based (ECC)**  
   - Uses `find_transform_ecc` to compute an affine or homography transform between frames.  
   - Simpler approach, can fail under large rotations or scale changes.

2. **Feature-Based**  
   - Detects corners/keypoints, tracks them via `calcOpticalFlowPyrLK`, and estimates a transform (`estimateAffine2D` or homography).  
   - More robust for bigger motion, assuming enough distinct features.

### 3. Preprocessing Steps

#### 3.1 Temporal or Spatial Denoising
1. **Spatial Denoising**  
   - If the raw frames contain sensor/camera noise, apply a spatial filter such as **Gaussian Blur**, **Median Blur**, or more advanced methods (e.g., **Non-Local Means**, `fastNlMeansDenoising`).  
   - Smooths out random speckle noise without overly blurring RBC edges.

2. **Temporal Filtering**  
   - Averaging or blending consecutive frames can reduce high-frequency jitter.  
   - For example, maintain a small buffer of past frames and perform a **temporal average** before stabilization or optical flow. This can help if the microscope’s frame rate is high enough.

#### 3.2 Vessel Masking & ROI Definition
Measuring RBC velocity across **entire frames** can introduce errors from non-vessel background and camera artifacts. We recommend:

1. **Vessel Detection**  
   - Apply morphological or advanced vesselness filters (e.g., **Frangi**, **Gabor**).  
   - Combine with thresholding (adaptive or Otsu) to isolate vessel-like structures.

2. **ROI Creation**  
   - If you only care about a specific region (e.g., a capillary bed), define a polygon or circular ROI, and combine it with your vessel mask.

3. **Masking for Optical Flow**  
   - Pass the binary mask (vessel=1, background=0) when computing **mean** flow magnitude.  
   - Alternatively, do a `bitwise_and` on stabilized frames so optical flow is computed only in vessel areas.

#### 3.3 Scale/Depth Change Handling (Z-Axis)
One tricky issue with **handheld microscopes** is **perpendicular motion** (moving closer/farther) that changes:
- **Vessel size** in the image.  
- Focus plane and sharpness.  

To partially address out-of-plane shifts:

1. **Homography Instead of Affine**  
   - In `find_transform_ecc`, set `motion_type = MOTION_HOMOGRAPHY`.  
   - Or in feature-based approaches, use `findHomography` instead of `estimateAffine2D`.  
   - A homography can handle moderate scale changes & perspective warping if not too extreme.

2. **Multi-Scale Approach**  
   - Downsample images for large-scale alignment first; warp the high-res frames next.  
   - If a user significantly changes the microscope’s distance, you may see large scale changes that 2D transforms cannot fully correct.

3. **3D or Depth Sensing**  
   - In advanced setups, a small **range sensor** or stereo approach can measure the actual Z-distance changes.  
   - Or at least track the microscope’s working distance (WD) with a mechanical gauge, and adjust your pixel-to-µm scale in real time.

Despite these steps, large or rapid perpendicular movements can still cause inconsistent vessel sizing or defocusing that no pure 2D method can fully rectify. Minimizing hand movement along the Z-axis in actual practice is often necessary.

---

### 4. RBC Velocity Estimation

1. **Stabilized (and optionally masked) Frames**  
   - Use `calc_optical_flow_farneback` (or another flow method) on consecutive stabilized frames.  
2. **Magnitude Calculation**  
   - Extract `(dx, dy)` flow, convert to magnitude with `cart_to_polar(dx, dy)`.  
3. **Mean Flow in ROI**  
   - If you have a vessel mask, call `core::mean(&mag, &mask)` to measure RBC flow only within vessels.  
4. **Physical Unit Conversion**  
   - If you know magnification (µm/pixel) and frame rate, you can compute RBC velocity in µm/s or mm/s.

### 5. Output & Visualization
- Display a **Stabilized Video** window, optionally overlaid with the vessel mask or ROI bounding shapes.  
- Print or log **per-frame RBC velocity** and a final **averaged** RBC velocity.  
- Additional advanced visualizations might include color-coded flow maps or time-series plots of RBC speed.

---

## Challenges & Considerations

### 1. Severe Handheld Microscopy Constraints
- Large out-of-plane motion or focus shifts cause **changes in vessel scale** and blur.  
- In practice, a **mechanical stabilizer** or careful brace drastically reduces 3D drift.

### 2. ECC vs Feature-Based Stabilization
- **ECC** alignment is simpler but often fails under large perspective changes.  
- **Feature-based** handles bigger transformations; still limited if the scene lacks distinct corners or if focus/lighting changes are large.

### 3. Advanced 2D/3D Compensation
- **Homography** can handle some scale and perspective changes better than affine.  
- True 3D motion requires more advanced solutions (e.g., stereo, depth sensors, or multi-camera rigs).

### 4. Parameter Tuning & Performance
- Farneback flow and morphological filters can be **compute-intensive**.  
- For near real-time, consider **GPU acceleration** (OpenCV CUDA), or reduce resolution.  
- Vessel segmentation filters (e.g., Frangi) also have **parameters** that require experimentation for best results.

---

## How to Run

### Dependencies

1. **Rust** (latest stable)  
2. **OpenCV** (installed on your system and matching the version used by the `opencv` crate)  
3. **Cargo.toml** with:
   ```toml
   [dependencies]
   opencv = "0.94.4"
   ```

### Build & Run

1. **Clone** this repository.  
2. Place your **video** file (e.g., `sample.mp4`) in the `./video/` directory. You can download sample videos from [this Google Drive folder](https://drive.google.com/drive/folders/1sGgIq-pUIRD9HfvFWnGl7GxU1OJUTglq?usp=drive_link).  
3. Adjust the `video_file` path in `main.rs`.  
4. **Build & run**:
   ```bash
   cargo run --release
   ```
5. A window titled **"Stabilized Video"** should appear, playing processed frames.  
6. Press **ESC** to exit at any time.

---

## References

1. **MicroTools Paper**  
   - [Automated Quantification of Capillary Density and RBC Velocity (Nature)](https://www.nature.com/articles/s42003-019-0473-8)  
   - Although we use Farneback flow, the MicroTools approach relies on correlation-based RBC velocity analysis. Their pipeline also includes advanced segmentation.  
2. **OpenCV Docs**  
   - [Farneback Optical Flow](https://docs.opencv.org/master/d7/d8b/group__video__motion.html)  
   - [ECC Image Alignment](https://docs.opencv.org/master/dc/d84/group__videostab.html)  
   - [Feature-Based Stabilization (`estimateAffine2D` or `findHomography`)](https://docs.opencv.org/master/d9/d0c/group__calib3d.html)  
3. **Vessel Segmentation Techniques**  
   - **Frangi Filter**, **Gabor Filters**, morphological ops, or simpler thresholding.  
   - Combining these can yield robust vessel masks for RBC flow analysis.  
4. **Depth & 3D**  
   - If perpendicular motion (Z-axis) is significant, consider hardware support or advanced 3D transformations to keep vessel size consistent.
