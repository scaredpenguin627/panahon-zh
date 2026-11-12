"""Create an original animated cloud-and-city sky loop for Panahon.

Run with:
  /workspace/blender-5.3/blender-5.3.0-linux-x64/blender \
    --background --python tools/blender_stage4_cloud_sky.py

The scene is intentionally self-contained: no Huawei assets, fonts, logos, or
external textures are used. It produces a vertical 540x960 PNG preview pair
and assets/blender_stage4/cloud_sky_scene.blend.
"""
from __future__ import annotations

import math
import os
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = ROOT / "assets" / "blender_stage4"
PREVIEW_DIR = ASSET_DIR / "preview"
BLEND_PATH = ASSET_DIR / "cloud_sky_scene.blend"


def mat_emission(name: str, color, strength: float = 1.0):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()
    out = nodes.new("ShaderNodeOutputMaterial")
    emit = nodes.new("ShaderNodeEmission")
    emit.inputs["Color"].default_value = (*color, 1.0)
    emit.inputs["Strength"].default_value = strength
    links.new(emit.outputs["Emission"], out.inputs["Surface"])
    return mat


def mat_principled(name: str, color, roughness: float = 0.8, metallic: float = 0.0):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    return mat


def sky_material():
    mat = bpy.data.materials.new("Original Sky Gradient")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()
    out = nodes.new("ShaderNodeOutputMaterial")
    emit = nodes.new("ShaderNodeEmission")
    tex = nodes.new("ShaderNodeTexCoord")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    ramp = nodes.new("ShaderNodeValToRGB")
    ramp.color_ramp.interpolation = "EASE"
    ramp.color_ramp.elements.remove(ramp.color_ramp.elements[1])
    stops = [
        (0.00, (0.035, 0.07, 0.18, 1.0)),
        (0.30, (0.13, 0.25, 0.48, 1.0)),
        (0.58, (0.44, 0.34, 0.55, 1.0)),
        (0.78, (0.90, 0.45, 0.28, 1.0)),
        (1.00, (0.98, 0.78, 0.39, 1.0)),
    ]
    for i, (pos, color) in enumerate(stops):
        elem = ramp.color_ramp.elements[0] if i == 0 else ramp.color_ramp.elements.new(pos)
        elem.position = pos
        elem.color = color
    links.new(tex.outputs["Generated"], sep.inputs["Vector"])
    links.new(sep.outputs["Y"], ramp.inputs["Fac"])
    links.new(ramp.outputs["Color"], emit.inputs["Color"])
    emit.inputs["Strength"].default_value = 0.75
    links.new(emit.outputs["Emission"], out.inputs["Surface"])
    return mat


def add_cube(name, location, scale, material, bevel=0.08):
    bpy.ops.mesh.primitive_cube_add(location=location)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        mod = obj.modifiers.new("Soft edges", "BEVEL")
        mod.width = bevel
        mod.segments = 3
    obj.data.materials.append(material)
    return obj


def add_cloud(parent, name, center, scale, material):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=1.0, location=center)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(material)
    obj.parent = parent
    return obj


def add_sky_plane():
    bpy.ops.mesh.primitive_plane_add(size=2, location=(0, 0, -0.3))
    plane = bpy.context.object
    plane.name = "Sky Gradient Backdrop"
    plane.scale = (14, 22, 1)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    plane.data.materials.append(sky_material())
    return plane


def build_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    ASSET_DIR.mkdir(parents=True, exist_ok=True)
    PREVIEW_DIR.mkdir(parents=True, exist_ok=True)

    scene = bpy.context.scene
    try:
        scene.render.engine = "BLENDER_EEVEE_NEXT"
    except TypeError:
        # This packaged Blender 5.3 build exposes the legacy Eevee enum.
        scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 360
    scene.render.resolution_y = 640
    scene.render.resolution_percentage = 100
    if hasattr(scene, "eevee"):
        scene.eevee.taa_render_samples = 8
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.film_transparent = False
    scene.render.filepath = str(PREVIEW_DIR / "cloud_sky_frame_")
    scene.render.fps = 24
    scene.frame_start = 1
    scene.frame_end = 120
    scene.render.image_settings.color_depth = "8"
    scene.view_settings.look = "AgX - Medium High Contrast"

    world = bpy.data.worlds.new("Original Twilight World")
    scene.world = world
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.018, 0.028, 0.08, 1.0)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.25

    add_sky_plane()

    camera_data = bpy.data.cameras.new("Cloud Sky Camera")
    camera = bpy.data.objects.new("Cloud Sky Camera", camera_data)
    scene.collection.objects.link(camera)
    camera.location = (0, 0, 20)
    camera.rotation_euler = (0, 0, 0)
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = 18.0
    scene.camera = camera

    # Soft key light for the skyline and cloud puffs.
    light_data = bpy.data.lights.new("Warm Sky Light", type="AREA")
    light_data.energy = 800
    light_data.shape = "DISK"
    light_data.size = 12
    light = bpy.data.objects.new("Warm Sky Light", light_data)
    light.location = (3.5, 4.5, 12)
    light.rotation_euler = (0, 0, 0)
    scene.collection.objects.link(light)

    sun_mat = mat_emission("Sun Disc", (1.0, 0.45, 0.16), 2.5)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=48, ring_count=24, radius=1.2, location=(4.1, 5.0, 1.0))
    sun = bpy.context.object
    sun.name = "Warm Sun Disc"
    sun.scale = (1.0, 1.0, 0.18)
    sun.data.materials.append(sun_mat)

    cloud_mat = mat_principled("Cloud Puffs", (0.90, 0.94, 1.0), roughness=0.92)
    cloud_mat.diffuse_color = (0.90, 0.94, 1.0, 1.0)
    cloud_mat.metallic = 0.0

    # Two independently animated cloud banks make the loop useful as a UI backdrop.
    banks = [
        ("Cloud Bank A", (-2.0, 4.0, 0.8), 1.0, 0.0),
        ("Cloud Bank B", (2.7, 1.8, 0.65), -1.0, 0.35),
    ]
    for name, center, direction, phase in banks:
        empty = bpy.data.objects.new(name, None)
        scene.collection.objects.link(empty)
        empty.location = center
        empty.keyframe_insert(data_path="location", frame=1)
        empty.location.x = center[0] + direction * 4.0
        empty.keyframe_insert(data_path="location", frame=61)
        empty.location.x = center[0]
        empty.keyframe_insert(data_path="location", frame=121)
        action = empty.animation_data.action
        # Blender 5.3 may expose layered actions without the legacy fcurves
        # collection; the default interpolation remains a smooth Bezier loop.
        if hasattr(action, "fcurves"):
            for fc in action.fcurves:
                for kp in fc.keyframe_points:
                    kp.interpolation = "SINE"
        puffs = [
            ((-1.6, 0.0, 0.0), (1.9, 0.82, 0.40)),
            ((-0.35, 0.35, 0.08), (1.55, 1.02, 0.52)),
            ((0.90, 0.18, 0.02), (1.85, 0.88, 0.43)),
            ((2.0, -0.08, -0.02), (1.3, 0.68, 0.34)),
        ]
        for idx, (offset, scale) in enumerate(puffs):
            add_cloud(empty, f"{name} Puff {idx + 1}", offset, scale, cloud_mat)

    # Original low-rise skyline silhouette at the bottom edge.
    building_mats = [
        mat_principled("Building Blue", (0.035, 0.08, 0.16), roughness=0.7),
        mat_principled("Building Slate", (0.06, 0.12, 0.20), roughness=0.75),
        mat_principled("Building Violet", (0.11, 0.10, 0.24), roughness=0.8),
    ]
    heights = [1.5, 2.5, 1.2, 3.5, 2.0, 4.2, 1.9, 2.8, 1.1, 3.0, 2.2]
    widths = [0.75, 0.65, 0.9, 0.8, 0.72, 0.9, 0.8, 0.95, 0.8, 0.7, 0.9]
    x = -7.0
    for idx, (height, width) in enumerate(zip(heights, widths)):
        y = -7.0 + (idx % 2) * 0.18
        add_cube(
            f"Skyline Building {idx + 1}",
            (x, y, height / 2 - 0.1),
            (width, 0.65, height / 2),
            building_mats[idx % len(building_mats)],
            bevel=0.06,
        )
        x += width * 2.0 + 0.28

    # Small warm window accents are emissive and deliberately generic.
    window_mat = mat_emission("Window Glow", (1.0, 0.42, 0.12), 1.4)
    for idx, wx in enumerate([-5.9, -3.6, -1.2, 1.6, 4.1, 5.9]):
        for row in range(2):
            bpy.ops.mesh.primitive_cube_add(location=(wx, -6.33, 0.45 + row * 0.58))
            win = bpy.context.object
            win.name = f"Window Accent {idx + 1}-{row + 1}"
            win.scale = (0.10, 0.025, 0.08)
            bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
            win.data.materials.append(window_mat)

    # Metadata helps later UI integration without binding the scene to app code.
    scene["panahon_stage"] = "stage4"
    scene["panahon_asset"] = "original_cloud_city_sky_loop"
    scene["panahon_loop_frames"] = 120
    scene["panahon_data_contract"] = "city_theme + cloudiness + visibility + quality"
    scene["panahon_clean_room"] = "original geometry/materials; no Huawei resources"

    scene.frame_set(1)
    scene.render.filepath = str(PREVIEW_DIR / "cloud_sky_frame_0001.png")
    bpy.ops.render.render(write_still=True)
    scene.frame_set(61)
    scene.render.filepath = str(PREVIEW_DIR / "cloud_sky_frame_0061.png")
    bpy.ops.render.render(write_still=True)
    scene.frame_set(1)
    bpy.ops.wm.save_as_mainfile(filepath=str(BLEND_PATH))
    print(f"STAGE4_BLEND={BLEND_PATH}")
    print(f"STAGE4_PREVIEW_1={PREVIEW_DIR / 'cloud_sky_frame_0001.png'}")
    print(f"STAGE4_PREVIEW_2={PREVIEW_DIR / 'cloud_sky_frame_0061.png'}")


if __name__ == "__main__":
    build_scene()
