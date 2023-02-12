// Todo lo que escribamos aquí puede ser invocado en app.rs

#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

use egui::mutex::Mutex;
use std::sync::Arc;

struct Util;

impl Util {

    fn visuales() -> egui::Visuals {
        let mut visual = egui::Visuals::dark();
        visual.panel_fill = egui::Color32::from_rgb(32, 33, 36);
        visual.override_text_color = Some(egui::Color32::from_rgb(5,213,255));
        //visual.override_text_color = Some(egui::Color32::WHITE);
        visual
    }

    fn tamano_fuente_adecuado(_cc: &eframe::CreationContext<'_>) -> f32 {
        // Obtención de un tamaño de letra que tiene en cuenta la resolución del monitor
        let puntos_fuente;
        
        #[cfg(target_arch = "wasm32")]
        {
        puntos_fuente = 20.0; // Tamaño de letra en versión web (provisional)
                            // Los puntos no parecen igual de grandes que en nativo
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
        let resolucion = _cc.integration_info.window_info.monitor_size.unwrap();
        //let resolucion = [1024.0, 768.0];
        let pixeles_por_punto = _cc.integration_info.native_pixels_per_point.unwrap();
        let pixeles_fuente_con_monitor_1080 = 20.0;
        let diagonal_monitor = ((resolucion[0] as f32).powi(2)  + (resolucion[1] as f32).powi(2)).sqrt();
        let diagonal_monitor_1080 = ((1920 as f32).powi(2)  + (1080 as f32).powi(2)).sqrt();
        let pixeles_fuente = (
            pixeles_fuente_con_monitor_1080 / diagonal_monitor_1080) * diagonal_monitor;
        puntos_fuente = pixeles_fuente / pixeles_por_punto;

        println!("Resolución de monitor: {}x{}", resolucion[0], resolucion[1]);
        println!("Píxeles por punto de forma nativa: {}", pixeles_por_punto);
        println!("Diagonal del monitor en píxeles: {}", diagonal_monitor);
        println!("Tamaño de fuente en píxeles: {}", pixeles_fuente);
        println!("Tamaño de fuente en puntos: {}", puntos_fuente);
        }

        puntos_fuente
    }
    
    fn cambiar_estilo_texto(cc: &eframe::CreationContext<'_>) {

        let mut fonts = egui::FontDefinitions::default();
        // Instalamos nuestra propia fuente (.ttf and .otf files supported)
        fonts.font_data.insert("fuente_1".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/Cantarell-VF.otf")),);
        fonts.font_data.insert("fuente_2".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/DroidSansMono-enMp.ttf")),);
        // Damos la máxima prioridad a nuestra fuente_1 para el texto "Proportional":
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "fuente_1".to_owned());
        // Damos la máxima prioridad a nuestra fuente_2 para el texto "Monospace":
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "fuente_2".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();
        let fuente = egui::FontFamily::Proportional;
        let puntos_fuente = Self::tamano_fuente_adecuado(cc);

        style.text_styles = [
            (egui::style::TextStyle::Heading, egui::FontId::new(puntos_fuente, fuente.clone())),
            (egui::style::TextStyle::Body, egui::FontId::new(puntos_fuente, fuente.clone())),
            (egui::style::TextStyle::Monospace, egui::FontId::new(puntos_fuente, fuente.clone())),
            (egui::style::TextStyle::Button, egui::FontId::new(puntos_fuente, fuente.clone())),
            (egui::style::TextStyle::Small, egui::FontId::new(puntos_fuente, fuente)),
        ].into();
        cc.egui_ctx.set_style(style);
    }

}




struct Dibujo {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<RotatingTriangle>>,
    angle: f32,
}


impl Dibujo {
    
    // Aquí se creará el RotatingTriangle con su método new
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl))),
            angle: 0.0,
        }
    }

    // Aquí se pintará el RotatingTriangle con su método paint
    fn graficacion(&mut self, ui: &mut egui::Ui) {

        // El rectángulo donde se graficará y el evento de ratón que se quiere captar en dicha región (arrastre)
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // Se cambiará el ángulo en función del arrastre introducido
        self.angle += response.drag_delta().x * 0.01;

        // Clonamos las vbles. angle y rotating_triangle para poder pasarlas en el paint callback
        let angle = self.angle;
        let rotating_triangle = self.rotating_triangle.clone();

        // Creación de un PaintCallback. rect es la región donde se pintará, y callback es un parámetro de tipo egui_glow::CallbackFn,
        // que contiene una función personalizada para pintar lo que queremos
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rotating_triangle.lock().paint(painter.gl(), angle);
            })),
        };
        ui.painter().add(callback);
    }

    // Aquí se destruirá el RotationTriangle con su método destroy
    fn destruir(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }

}


// Aquí se define todo lo que necesita OpenGL para dibujar un objeto RotationTriangle en un área dada
// (creación de buffers de vértices, shaders, llamadas para pintar y destruir...)
struct RotatingTriangle {
    program: glow::Program,
    vertex_array: glow::VertexArray,
}

impl RotatingTriangle {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                    const vec2 verts[3] = vec2[3](
                        vec2(0.0, 1.0),
                        vec2(-1.0, -1.0),
                        vec2(1.0, -1.0)
                    );
                    const vec4 colors[3] = vec4[3](
                        vec4(1.0, 0.0, 0.0, 1.0),
                        vec4(0.0, 1.0, 0.0, 1.0),
                        vec4(0.0, 0.0, 1.0, 1.0)
                    );
                    out vec4 v_color;
                    uniform float u_angle;
                    void main() {
                        v_color = colors[gl_VertexID];
                        gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
                        gl_Position.x *= cos(u_angle);
                    }
                "#,
                r#"
                    precision mediump float;
                    in vec4 v_color;
                    out vec4 out_color;
                    void main() {
                        out_color = v_color;
                    }
                "#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Self {
                program,
                vertex_array,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn paint(&self, gl: &glow::Context, angle: f32) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_angle").as_ref(),
                angle,
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}