use macroquad::prelude::*;

const GRAVIDADE: f32 = 600.0;
const RESTITUICAO: f32 = 0.8;
const ATRITO: f32 = 0.99;

struct Particula {
    posicao: Vec2,
    velocidade: Vec2,
    raio: f32,
    cor: Color,
}

impl Particula {
    fn atualizar(&mut self, dt: f32, limites: Rect) {
        self.velocidade.y += GRAVIDADE * dt;
        self.posicao += self.velocidade * dt;

        if self.posicao.x - self.raio < limites.x {
            self.posicao.x = limites.x + self.raio;
            self.velocidade.x = -self.velocidade.x * RESTITUICAO;
            self.velocidade.y *= ATRITO;
        }
        if self.posicao.x + self.raio > limites.x + limites.w {
            self.posicao.x = limites.x + limites.w - self.raio;
            self.velocidade.x = -self.velocidade.x * RESTITUICAO;
            self.velocidade.y *= ATRITO;
        }
        if self.posicao.y - self.raio < limites.y {
            self.posicao.y = limites.y + self.raio;
            self.velocidade.y = -self.velocidade.y * RESTITUICAO;
            self.velocidade.x *= ATRITO;
        }
        if self.posicao.y + self.raio > limites.y + limites.h {
            self.posicao.y = limites.y + limites.h - self.raio;
            self.velocidade.y = -self.velocidade.y * RESTITUICAO;
            self.velocidade.x *= ATRITO;
        }
    }
}

fn particula_aleatoria(limites: Rect) -> Particula {
    let raio = rand::gen_range(3.0, 6.0);
    let margem = 20.0;

    let posicao = vec2(
        rand::gen_range(limites.x + margem, limites.x + limites.w - margem),
        rand::gen_range(limites.y + margem, limites.y + limites.h - margem),
    );

    let velocidade = vec2(
        rand::gen_range(-200.0, 200.0),
        rand::gen_range(-50.0, 0.0),
    );

    let cor = Color::new(
        rand::gen_range(0.2, 1.0),
        rand::gen_range(0.2, 1.0),
        rand::gen_range(0.2, 1.0),
        1.0,
    );

    Particula { posicao, velocidade, raio, cor }
}

#[macroquad::main("Partículas")]
async fn main() {
    let mut particulas: Vec<Particula> = Vec::new();
    let mut semente_feita = false;

    loop {
        let limites = Rect::new(0.0, 0.0, screen_width(), screen_height());

        if !semente_feita {
            particulas = (0..250).map(|_| particula_aleatoria(limites)).collect();
            semente_feita = true;
        }

        let dt = get_frame_time().min(1.0 / 30.0);

        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let mut p = particula_aleatoria(limites);
            p.posicao = vec2(mx, my);
            particulas.push(p);
        }

        if is_mouse_button_down(MouseButton::Right) {
            let cursor = vec2(mouse_position().0, mouse_position().1);
            for p in &mut particulas {
                let ate_p = p.posicao - cursor;
                let dist2 = ate_p.length_squared().max(1.0);
                let dir = if dist2 > 0.0 { ate_p.normalize() } else { vec2(0.0, 0.0) };
                let impulso = 120000.0 / dist2;
                p.velocidade += dir * impulso * dt * 2.0;
            }
        }

        for p in &mut particulas {
            p.atualizar(dt, limites);
        }

        clear_background(BLACK);

        for p in &particulas {
            draw_circle(p.posicao.x, p.posicao.y, p.raio, p.cor);
        }

        draw_text(
            &format!("Partículas: {} | Esq: criar | Dir: corrente", particulas.len()),
            16.0, 24.0, 24.0, GRAY,
        );

        next_frame().await;
    }
}
