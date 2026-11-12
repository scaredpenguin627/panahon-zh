# Stage 4 Blender cloud-sky prototype

`cloud_sky_scene.blend` is an original, self-contained vertical sky loop. It uses procedural geometry and materials only; it contains no Huawei fonts, images, logos, trademarks, or code.

- Blender: 5.3.0
- Render: Eevee Next, 360×640 RGBA PNG
- Loop: 120 frames at 24 fps (5 seconds)
- Preview frames: `preview/cloud_sky_frame_0001.png` and `preview/cloud_sky_frame_0061.png`
- Generator: `tools/blender_stage4_cloud_sky.py`

The animation is a presentation prototype. Runtime weather values must remain UI data: a future egui integration should select the sky theme and cloud layer from `Dashboard`/`DataQuality`, while the PNG or a lightweight procedural fallback is only a visual layer.

## egui fallback

If Blender output is unavailable, draw a vertical two- or three-stop gradient with `egui::Painter`, add two translucent rounded cloud groups and interpolate their x-offset over a five-second loop. Keep the city, cloudiness, visibility, quality, source, and update time as text/UI data above the visual layer. Do not treat the rendered PNG as a weather observation.

## Rebuild

```sh
/workspace/blender-5.3/blender-5.3.0-linux-x64/blender \
  --background --python tools/blender_stage4_cloud_sky.py
```
