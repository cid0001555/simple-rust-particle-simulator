mod ambiente;
mod cenas;
mod fisica;
mod grade_espacial;
mod interface;
mod materiais;
mod particula;

use macroquad::prelude::*;

use ambiente::FerramentaAtiva;
use fisica::MundoSimulacao;
use interface::{
    ajustar_ponto_parede, desenhar_e_interagir_interface, desenhar_mundo, AcaoInterface,
    EstadoInterface, Tipografia,
};
use materiais::MaterialParticula;

fn configuracao_janela() -> Conf {
    Conf {
        window_title: "Partículas".to_string(),
        window_width: 1280,
        window_height: 780,
        sample_count: 4,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

fn processar_atalhos_teclado(
    mundo: &mut MundoSimulacao,
    interface: &mut EstadoInterface,
    cursor: Vec2,
    limites_simulacao: Rect,
) {
    if is_key_pressed(KeyCode::Key1) {
        interface.selecionar_material(MaterialParticula::Elastico);
    }
    if is_key_pressed(KeyCode::Key2) {
        interface.selecionar_material(MaterialParticula::Agua);
    }
    if is_key_pressed(KeyCode::Key3) {
        interface.selecionar_material(MaterialParticula::Areia);
    }
    if is_key_pressed(KeyCode::Key4) {
        interface.selecionar_material(MaterialParticula::Gas);
    }
    if is_key_pressed(KeyCode::Key5) {
        interface.selecionar_material(MaterialParticula::Fogo);
    }
    if is_key_pressed(KeyCode::Key6) {
        interface.selecionar_material(MaterialParticula::Magnetico);
    }
    if is_key_pressed(KeyCode::Key7) {
        interface.selecionar_ferramenta(FerramentaAtiva::Parede);
    }
    if is_key_pressed(KeyCode::Key8) {
        interface.selecionar_ferramenta(FerramentaAtiva::Ancora);
    }
    if is_key_pressed(KeyCode::Key9) || is_key_pressed(KeyCode::E) {
        interface.selecionar_ferramenta(FerramentaAtiva::Fonte);
    }
    if is_key_pressed(KeyCode::Key0) || is_key_pressed(KeyCode::Backspace) {
        interface.selecionar_ferramenta(FerramentaAtiva::Borracha);
    }

    if is_key_pressed(KeyCode::F) {
        mundo.modo_forca = mundo.modo_forca.proximo();
    }
    if is_key_pressed(KeyCode::G) {
        mundo.modo_gravidade = mundo.modo_gravidade.proximo();
    }
    if is_key_pressed(KeyCode::V) {
        mundo.modo_visual = mundo.modo_visual.proximo();
    }
    if is_key_pressed(KeyCode::C) {
        interface.cena_atual = interface.cena_atual.proxima();
        interface.cena_atual.aplicar(mundo, limites_simulacao);
    }
    if is_key_pressed(KeyCode::R) {
        interface.cena_atual.aplicar(mundo, limites_simulacao);
    }
    if is_key_pressed(KeyCode::X) {
        mundo.limpar_tudo();
    }
    if is_key_pressed(KeyCode::Delete) {
        mundo.limpar_particulas();
    }
    if is_key_pressed(KeyCode::Space) {
        mundo.pausado = !mundo.pausado;
    }
    if is_key_pressed(KeyCode::Period) && mundo.pausado {
        mundo.atualizar_passo(1.0 / 60.0, limites_simulacao);
    }
    if is_key_pressed(KeyCode::Q) {
        mundo.disparar_onda_choque(cursor, interface.raio_pincel);
    }
    if is_key_pressed(KeyCode::Tab) {
        interface.barra_visivel = !interface.barra_visivel;
    }
    if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::F1) {
        interface.mostrar_ajuda = !interface.mostrar_ajuda;
    }
    if is_key_pressed(KeyCode::Escape) {
        interface.mostrar_ajuda = false;
        interface.inicio_parede = None;
        interface.inicio_fonte = None;
    }
}

fn aplicar_acao_interface(
    acao: AcaoInterface,
    mundo: &mut MundoSimulacao,
    interface: &mut EstadoInterface,
    limites_simulacao: Rect,
) {
    match acao {
        AcaoInterface::Nenhuma => {}
        AcaoInterface::SelecionarFerramenta(ferramenta) => {
            interface.selecionar_ferramenta(ferramenta);
        }
        AcaoInterface::AlternarForca => {
            mundo.modo_forca = mundo.modo_forca.proximo();
        }
        AcaoInterface::AlternarGravidade => {
            mundo.modo_gravidade = mundo.modo_gravidade.proximo();
        }
        AcaoInterface::AlternarVisual => {
            mundo.modo_visual = mundo.modo_visual.proximo();
        }
        AcaoInterface::ProximaCena => {
            interface.cena_atual = interface.cena_atual.proxima();
            interface.cena_atual.aplicar(mundo, limites_simulacao);
        }
        AcaoInterface::AlternarAjuda => {
            interface.mostrar_ajuda = !interface.mostrar_ajuda;
        }
    }
}

#[macroquad::main(configuracao_janela)]
async fn main() {
    let tipografia = Tipografia::carregar();
    let mut mundo = MundoSimulacao::novo();
    let mut interface = EstadoInterface::novo();
    let mut cena_inicial_carregada = false;

    loop {
        let delta_tempo = get_frame_time().clamp(1.0 / 240.0, 1.0 / 30.0);
        let (cursor_x, cursor_y) = mouse_position();
        let cursor = vec2(cursor_x, cursor_y);
        let velocidade_cursor = (cursor - interface.cursor_anterior) / delta_tempo;

        let (_, rolagem_y) = mouse_wheel();
        if rolagem_y.abs() > 0.001 {
            let passo_raio = rolagem_y.signum() * 4.0;
            interface.raio_pincel = (interface.raio_pincel + passo_raio).clamp(8.0, 96.0);
        }

        let limites_simulacao = interface.limites_simulacao(screen_width(), screen_height());
        if !cena_inicial_carregada {
            interface.cena_atual.aplicar(&mut mundo, limites_simulacao);
            cena_inicial_carregada = true;
        }

        processar_atalhos_teclado(&mut mundo, &mut interface, cursor, limites_simulacao);

        let clique_esquerdo_inicial = is_mouse_button_pressed(MouseButton::Left);
        let sobre_interface = interface.cursor_sobre_interface(cursor);
        let alinhar_parede = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        if !sobre_interface {
            match interface.ferramenta {
                FerramentaAtiva::Material(material) => {
                    if is_mouse_button_down(MouseButton::Left) {
                        let quantidade = ((interface.raio_pincel / 9.0).ceil() as usize).clamp(1, 7);
                        mundo.criar_pincel(
                            cursor,
                            interface.raio_pincel,
                            material,
                            quantidade,
                            limites_simulacao,
                        );
                    }
                }
                FerramentaAtiva::Parede => {
                    if clique_esquerdo_inicial {
                        interface.inicio_parede = Some(cursor);
                    }
                    if is_mouse_button_released(MouseButton::Left)
                        && let Some(inicio) = interface.inicio_parede.take()
                    {
                        let fim = ajustar_ponto_parede(inicio, cursor, alinhar_parede);
                        mundo.adicionar_obstaculo(inicio, fim, 5.5);
                    }
                }
                FerramentaAtiva::Ancora => {
                    if clique_esquerdo_inicial {
                        mundo.alternar_ancora(cursor, interface.raio_pincel);
                    }
                }
                FerramentaAtiva::Fonte => {
                    if clique_esquerdo_inicial && !mundo.remover_emissor_proximo(cursor) {
                        interface.inicio_fonte = Some(cursor);
                    }
                    if is_mouse_button_released(MouseButton::Left)
                        && let Some(origem_fonte) = interface.inicio_fonte.take()
                    {
                        mundo.adicionar_emissor(origem_fonte, Some(cursor), interface.ultimo_material);
                    }
                }
                FerramentaAtiva::Borracha => {
                    if is_mouse_button_down(MouseButton::Left) {
                        mundo.apagar_em(cursor, interface.raio_pincel);
                    }
                }
            }

            if is_mouse_button_down(MouseButton::Right) {
                mundo.aplicar_forca_cursor(cursor, velocidade_cursor, interface.raio_pincel, delta_tempo);
            }
            if is_mouse_button_pressed(MouseButton::Middle) {
                mundo.disparar_onda_choque(cursor, interface.raio_pincel);
            }
        } else if is_mouse_button_released(MouseButton::Left) {
            interface.inicio_parede = None;
            interface.inicio_fonte = None;
        }

        if !mundo.pausado {
            mundo.atualizar_passo(delta_tempo, limites_simulacao);
        }
        mundo.atualizar_efeitos_visuais(delta_tempo);

        desenhar_mundo(&mundo, &interface, cursor, alinhar_parede);
        let acao = desenhar_e_interagir_interface(
            &mut interface,
            &mundo,
            &tipografia,
            cursor,
            clique_esquerdo_inicial,
        );
        aplicar_acao_interface(acao, &mut mundo, &mut interface, limites_simulacao);

        interface.cursor_anterior = cursor;
        next_frame().await;
    }
}
