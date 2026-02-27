#![windows_subsystem = "windows"]
use app::{app, AppArgs, NAME};
use chargrid_wgpu::*;
use native::{meap, NativeCommon};

struct Args {
    native_common: NativeCommon,
    force_opengl: bool,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                native_common = NativeCommon::parser();
                force_opengl = flag("force-opengl").desc("force opengl");
            } in {
                Self { native_common, force_opengl }
            }
        }
    }
}

fn main() {
    use meap::Parser;
    env_logger::init();
    let Args {
        native_common:
            NativeCommon {
                storage,
                initial_rng_seed,
                omniscient,
                new_game,
            },
        force_opengl,
    } = Args::parser().with_help_default().parse_env_or_exit();
    let app = app(AppArgs {
        storage,
        initial_rng_seed,
        omniscient,
        new_game,
    });
    run(
        app,
        Config {
            title: NAME.to_string(),
            dimensions_px: Dimensions {
                width: 960.,
                height: 720.,
            },
            resizable: false,
            font_bytes: FontBytes::new(
                include_bytes!("./fonts/PxPlus_IBM_CGAthin-2y.ttf").to_vec(),
                include_bytes!("./fonts/PxPlus_IBM_CGA-2y.ttf").to_vec(),
            ),
            cell_dimensions_px: Dimensions {
                width: 12.,
                height: 24.,
            },
            character_cell_offset_px: Dimensions {
                width: 0.,
                height: 2.,
            },
            font_size_px: 24.,
            underline_width_cell_ratio: 0.1,
            underline_top_offset_cell_ratio: 0.8,
            force_secondary_adapter: force_opengl,
        },
    )
    .unwrap();
}
