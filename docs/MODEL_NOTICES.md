# Model notices

AirControl bundles two ONNX models from the OpenCV Zoo MediaPipe hand pipeline:

- `palm_detection_mediapipe_2023feb.onnx` detects palm regions.
- `handpose_estimation_mediapipe_2023feb.onnx` estimates 21 hand landmarks for a detected hand crop.

Both model directories state that their contents are licensed under Apache License 2.0. The accompanying [`LICENSE-APACHE-2.0.txt`](../src-tauri/models/LICENSE-APACHE-2.0.txt) is included with the bundled weights. The model files are loaded from the local application binary; runtime model download and image upload are not used.

Upstream model documentation:

- [OpenCV Zoo palm detection](https://github.com/opencv/opencv_zoo/tree/main/models/palm_detection_mediapipe)
- [OpenCV Zoo hand pose estimation](https://github.com/opencv/opencv_zoo/tree/main/models/handpose_estimation_mediapipe)
- [OpenCV Zoo repository](https://github.com/opencv/opencv_zoo)

Model SHA-256 values for this checkout:

| Model | SHA-256 |
| --- | --- |
| Palm detector | `78ff51c38496b7fc8b8ebdb6cc8c1abb02fa6c38427c6848254cdaba57fcce7c` |
| Hand landmarks | `db0898ae717b76b075d9bf563af315b29562e11f8df5027a1ef07b02bef6d81c` |

The model weights are provided for local inference. This notice does not claim that the real-time tracking pipeline, model accuracy, or hardware performance has been validated on a Windows PC.
