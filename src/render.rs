use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use crate::{
    asset_loader::ImposterLoader,
    oct_coords::{GridMode, GRID_MASK},
};

pub const BINDINGS_HANDLE: Handle<Shader> = Handle::weak_from_u128(659996873659996873);
pub const FRAGMENT_HANDLE: Handle<Shader> = Handle::weak_from_u128(656126482580442360);
pub const SHARED_HANDLE: Handle<Shader> = Handle::weak_from_u128(699899997614446892);
pub const VERTEX_HANDLE: Handle<Shader> = Handle::weak_from_u128(591046068481766317);

pub const VERTEX_BILLBOARD_FLAG: u32 = 4;
pub const RENDER_MULTISAMPLE_FLAG: u32 = 16;
pub const USE_SOURCE_UV_Y_FLAG: u32 = 8;

pub struct ImposterRenderPlugin;

impl Plugin for ImposterRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            BINDINGS_HANDLE,
            "shaders/bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            FRAGMENT_HANDLE,
            "shaders/fragment.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, SHARED_HANDLE, "shaders/shared.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, VERTEX_HANDLE, "shaders/vertex.wgsl", Shader::from_wgsl);

        app.add_plugins(MaterialPlugin::<Imposter>::default())
            .register_asset_loader(ImposterLoader);
    }
}

#[derive(ShaderType, Clone, Copy, PartialEq, Debug)]
pub struct ImposterData {
    pub center_and_scale: Vec4,
    pub grid_size: u32,
    pub flags: u32,
}

impl ImposterData {
    pub fn new(
        center: Vec3,
        scale: f32,
        grid_size: u32,
        mode: GridMode,
        billboard_vertices: bool,
        multisample: bool,
        use_mesh_uv_y: bool,
    ) -> Self {
        Self {
            center_and_scale: center.extend(scale),
            grid_size,
            flags: mode.as_flags()
                + if billboard_vertices {
                    VERTEX_BILLBOARD_FLAG
                } else {
                    0
                }
                + if multisample {
                    RENDER_MULTISAMPLE_FLAG
                } else {
                    0
                }
                + if use_mesh_uv_y {
                    USE_SOURCE_UV_Y_FLAG
                } else {
                    0
                },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ImposterKey(u32);

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ImposterKey)]
pub struct Imposter {
    #[uniform(200)]
    pub data: ImposterData,
    #[texture(201, dimension = "2d", sample_type = "u_int")]
    pub material: Option<Handle<Image>>,
}

impl From<&Imposter> for ImposterKey {
    fn from(value: &Imposter) -> Self {
        Self(value.data.flags)
    }
}

impl Material for Imposter {
    fn vertex_shader() -> ShaderRef {
        VERTEX_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        FRAGMENT_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vert_defs = &mut descriptor.vertex.shader_defs;
        let frag_defs = &mut descriptor.fragment.as_mut().unwrap().shader_defs;

        if (key.bind_group_data.0 & RENDER_MULTISAMPLE_FLAG) != 0 {
            frag_defs.push("MATERIAL_MULTISAMPLE".into());
        }
        if (key.bind_group_data.0 & VERTEX_BILLBOARD_FLAG) != 0 {
            vert_defs.push("VERTEX_BILLBOARD".into());
        }
        if (key.bind_group_data.0 & USE_SOURCE_UV_Y_FLAG) != 0 {
            vert_defs.push("USE_SOURCE_UV_Y".into());
            frag_defs.push("USE_SOURCE_UV_Y".into());
        }
        let grid_mode = match key.bind_group_data.0 & GRID_MASK {
            i if i == GridMode::Hemispherical.as_flags() => "GRID_HEMISPHERICAL",
            i if i == GridMode::Spherical.as_flags() => "GRID_SPHERICAL",
            i if i == GridMode::Horizontal.as_flags() => "GRID_HORIZONTAL",
            _ => panic!(),
        };
        vert_defs.push(grid_mode.into());
        frag_defs.push(grid_mode.into());

        Ok(())
    }
}
