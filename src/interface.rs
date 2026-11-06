use macroquad::prelude::*;

use crate::ambiente::{FerramentaAtiva, ModoVisual};
use crate::cenas::Cena;
use crate::fisica::MundoSimulacao;
use crate::materiais::{cor_por_velocidade, MaterialParticula};

pub struct Tipografia {
    fonte: Option<Font>,
}

impl Tipografia {
    pub fn carregar() -> Self {
        let caminhos_candidatos = [
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ];

        let mut fonte_carregada = None;
        for caminho in caminhos_candidatos {
            if let Ok(bytes_arquivo) = std::fs::read(caminho)
                && let Ok(fonte_valida) = load_ttf_font_from_bytes(&bytes_arquivo)
            {
                fonte_carregada = Some(fonte_valida);
                break;
            }
        }

        Self {
            fonte: fonte_carregada,
        }
    }

    pub fn medir(&self, texto: &str, tamanho: u16) -> TextDimensions {
        if let Some(ref fonte_interna) = self.fonte {
            measure_text(texto, Some(fonte_interna), tamanho, 1.0)
        } else {
            let sem_acento = remover_acentos(texto);
            measure_text(&sem_acento, None, tamanho, 1.0)
        }
    }

    pub fn desenhar(&self, texto: &str, posicao_x: f32, posicao_y: f32, tamanho: u16, cor: Color) {
        if let Some(ref fonte_interna) = self.fonte {
            draw_text_ex(
                texto,
                posicao_x.round(),
                posicao_y.round(),
                TextParams {
                    font: Some(fonte_interna),
                    font_size: tamanho,
                    font_scale: 1.0,
                    color: cor,
                    ..Default::default()
                },
            );
        } else {
            let sem_acento = remover_acentos(texto);
            draw_text(&sem_acento, posicao_x.round(), posicao_y.round(), tamanho as f32, cor);
        }
    }
}

fn remover_acentos(entrada: &str) -> String {
    entrada
        .chars()
        .map(|caractere| match caractere {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'Á' | 'À' | 'Â' | 'Ã' => 'A',
            'é' | 'ê' => 'e',
            'É' | 'Ê' => 'E',
            'í' => 'i',
            'Í' => 'I',
            'ó' | 'ô' | 'õ' => 'o',
            'Ó' | 'Ô' | 'Õ' => 'O',
            'ú' | 'ü' => 'u',
            'Ú' | 'Ü' => 'U',
            'ç' => 'c',
            'Ç' => 'C',
            '·' => '|',
            '–' => '-',
            outro => outro,
        })
        .collect()
}

pub enum AcaoInterface {
    Nenhuma,
    SelecionarFerramenta(FerramentaAtiva),
    AlternarForca,
    AlternarGravidade,
    AlternarVisual,
    ProximaCena,
    AlternarAjuda,
}

pub struct EstadoInterface {
    pub ferramenta: FerramentaAtiva,
    pub ultimo_material: MaterialParticula,
    pub cena_atual: Cena,
    pub raio_pincel: f32,
    pub barra_visivel: bool,
    pub mostrar_ajuda: bool,
    pub altura_barra_atual: f32,
    pub inicio_parede: Option<Vec2>,
    pub inicio_fonte: Option<Vec2>,
    pub cursor_anterior: Vec2,
}

impl EstadoInterface {
    pub fn novo() -> Self {
        Self {
            ferramenta: FerramentaAtiva::Material(MaterialParticula::Elastico),
            ultimo_material: MaterialParticula::Elastico,
            cena_atual: Cena::Livre,
            raio_pincel: 26.0,
            barra_visivel: true,
            mostrar_ajuda: false,
            altura_barra_atual: 42.0,
            inicio_parede: None,
            inicio_fonte: None,
            cursor_anterior: Vec2::ZERO,
        }
    }

    pub fn selecionar_material(&mut self, material: MaterialParticula) {
        self.ultimo_material = material;
        self.ferramenta = FerramentaAtiva::Material(material);
        self.inicio_parede = None;
        self.inicio_fonte = None;
    }

    pub fn selecionar_ferramenta(&mut self, nova_ferramenta: FerramentaAtiva) {
        if let FerramentaAtiva::Material(material) = nova_ferramenta {
            self.ultimo_material = material;
        }
        self.ferramenta = nova_ferramenta;
        self.inicio_parede = None;
        self.inicio_fonte = None;
    }

    pub fn limites_simulacao(&self, largura_tela: f32, altura_tela: f32) -> Rect {
        let topo = if self.barra_visivel {
            self.altura_barra_atual
        } else {
            0.0
        };
        Rect::new(0.0, topo, largura_tela.max(64.0), (altura_tela - topo).max(64.0))
    }

    pub fn cursor_sobre_interface(&self, cursor: Vec2) -> bool {
        self.mostrar_ajuda || (self.barra_visivel && cursor.y <= self.altura_barra_atual)
    }
}

pub fn ajustar_ponto_parede(inicio: Vec2, fim: Vec2, alinhar_eixos: bool) -> Vec2 {
    if !alinhar_eixos {
        return fim;
    }
    let diferenca = fim - inicio;
    let comprimento = diferenca.length();
    if comprimento < 2.0 {
        return fim;
    }
    let angulo = diferenca.y.atan2(diferenca.x);
    let passo_angulo = std::f32::consts::FRAC_PI_4;
    let angulo_ajustado = (angulo / passo_angulo).round() * passo_angulo;
    inicio + vec2(angulo_ajustado.cos(), angulo_ajustado.sin()) * comprimento
}

pub fn desenhar_mundo(
    mundo: &MundoSimulacao,
    interface: &EstadoInterface,
    cursor: Vec2,
    alinhar_parede: bool,
) {
    clear_background(Color::new(0.035, 0.042, 0.062, 1.0));

    for ancora in &mundo.ancoras {
        let cor_base = ancora.modo.cor();
        draw_circle_lines(
            ancora.posicao.x,
            ancora.posicao.y,
            ancora.raio_acao,
            1.0,
            Color::new(cor_base.r, cor_base.g, cor_base.b, 0.14),
        );
        let pulso = 0.5 + 0.5 * ancora.fase_animacao.sin();
        let raio_interno = 11.0 + pulso * 7.0;
        draw_circle_lines(
            ancora.posicao.x,
            ancora.posicao.y,
            raio_interno,
            1.6,
            Color::new(cor_base.r, cor_base.g, cor_base.b, 0.55),
        );
        draw_circle(
            ancora.posicao.x,
            ancora.posicao.y,
            5.0,
            Color::new(cor_base.r, cor_base.g, cor_base.b, 0.92),
        );
    }

    for emissor in &mundo.emissores {
        let cor_fonte = emissor.material.cor_icone();
        let ponta = emissor.posicao + emissor.direcao * 14.0;
        draw_line(
            emissor.posicao.x,
            emissor.posicao.y,
            ponta.x,
            ponta.y,
            3.0,
            Color::new(cor_fonte.r, cor_fonte.g, cor_fonte.b, 0.75),
        );
        draw_circle(
            emissor.posicao.x,
            emissor.posicao.y,
            6.5,
            Color::new(0.14, 0.16, 0.22, 1.0),
        );
        draw_circle_lines(emissor.posicao.x, emissor.posicao.y, 6.5, 2.0, cor_fonte);
    }

    let cor_obstaculo = Color::new(0.28, 0.34, 0.46, 1.0);
    let cor_nucleo_obstaculo = Color::new(0.40, 0.48, 0.62, 1.0);
    for obstaculo in &mundo.obstaculos {
        desenhar_capsula_solida(
            obstaculo.inicio,
            obstaculo.fim,
            obstaculo.espessura,
            cor_obstaculo,
        );
        desenhar_capsula_solida(
            obstaculo.inicio,
            obstaculo.fim,
            (obstaculo.espessura * 0.45).max(1.5),
            cor_nucleo_obstaculo,
        );
    }

    for particula in &mundo.particulas {
        let rapidez = particula.velocidade.length();
        let cor_principal = match mundo.modo_visual {
            ModoVisual::Material | ModoVisual::Rastro => particula.cor_renderizacao(),
            ModoVisual::Velocidade => cor_por_velocidade(rapidez),
        };

        if mundo.modo_visual == ModoVisual::Rastro {
            let deslocamento = particula.posicao - particula.posicao_anterior;
            if deslocamento.length_squared() > 0.5 {
                let cauda = particula.posicao - deslocamento * 2.2;
                draw_line(
                    cauda.x,
                    cauda.y,
                    particula.posicao.x,
                    particula.posicao.y,
                    (particula.raio * 1.15).max(1.5),
                    Color::new(cor_principal.r, cor_principal.g, cor_principal.b, 0.34),
                );
            }
        }

        let propriedades = particula.material.propriedades();
        if propriedades.emissivo || (mundo.modo_visual == ModoVisual::Velocidade && rapidez > 380.0) {
            draw_circle(
                particula.posicao.x,
                particula.posicao.y,
                particula.raio * 2.0,
                Color::new(cor_principal.r, cor_principal.g, cor_principal.b, 0.14),
            );
        }

        draw_circle(
            particula.posicao.x,
            particula.posicao.y,
            particula.raio,
            cor_principal,
        );
    }

    for onda in &mundo.ondas {
        let opacidade = (onda.vida_restante / onda.vida_total.max(0.001)).clamp(0.0, 1.0) * 0.78;
        draw_circle_lines(
            onda.centro.x,
            onda.centro.y,
            onda.raio_atual,
            2.2,
            Color::new(onda.cor.r, onda.cor.g, onda.cor.b, opacidade),
        );
    }

    if !interface.cursor_sobre_interface(cursor) {
        if let Some(inicio) = interface.inicio_parede {
            let fim = ajustar_ponto_parede(inicio, cursor, alinhar_parede);
            desenhar_capsula_solida(inicio, fim, 5.5, Color::new(0.45, 0.68, 0.96, 0.72));
        } else if let Some(origem_fonte) = interface.inicio_fonte {
            let cor_fonte = interface.ultimo_material.cor_icone();
            draw_line(
                origem_fonte.x,
                origem_fonte.y,
                cursor.x,
                cursor.y,
                2.5,
                Color::new(cor_fonte.r, cor_fonte.g, cor_fonte.b, 0.75),
            );
            draw_circle_lines(origem_fonte.x, origem_fonte.y, 7.0, 2.0, cor_fonte);
        } else {
            let cor_cursor = if is_mouse_button_down(MouseButton::Right) {
                mundo.modo_forca.cor()
            } else {
                match interface.ferramenta {
                    FerramentaAtiva::Material(material) => material.cor_icone(),
                    FerramentaAtiva::Parede => Color::new(0.62, 0.74, 0.92, 1.0),
                    FerramentaAtiva::Ancora => mundo.modo_forca.cor(),
                    FerramentaAtiva::Fonte => interface.ultimo_material.cor_icone(),
                    FerramentaAtiva::Borracha => Color::new(0.96, 0.42, 0.42, 1.0),
                }
            };

            let raio_visual = if is_mouse_button_down(MouseButton::Right) {
                (interface.raio_pincel * 3.4).max(95.0)
            } else if matches!(
                interface.ferramenta,
                FerramentaAtiva::Parede | FerramentaAtiva::Ancora | FerramentaAtiva::Fonte
            ) {
                9.0
            } else {
                interface.raio_pincel
            };

            draw_circle_lines(
                cursor.x,
                cursor.y,
                raio_visual,
                1.3,
                Color::new(cor_cursor.r, cor_cursor.g, cor_cursor.b, 0.55),
            );
        }
    }
}

fn desenhar_capsula_solida(inicio: Vec2, fim: Vec2, raio: f32, cor: Color) {
    draw_line(inicio.x, inicio.y, fim.x, fim.y, raio * 2.0, cor);
    draw_circle(inicio.x, inicio.y, raio, cor);
    draw_circle(fim.x, fim.y, raio, cor);
}

fn preencher_capsula_retangular(retangulo: Rect, cor: Color) {
    let raio = retangulo.h * 0.5;
    let largura_interna = (retangulo.w - retangulo.h).max(0.0);
    draw_circle(retangulo.x + raio, retangulo.y + raio, raio, cor);
    draw_circle(retangulo.x + retangulo.w - raio, retangulo.y + raio, raio, cor);
    draw_rectangle(retangulo.x + raio, retangulo.y, largura_interna, retangulo.h, cor);
}

fn desenhar_pilula_opaca(retangulo: Rect, cor_fundo: Color, cor_borda: Option<Color>) {
    if let Some(borda) = cor_borda {
        preencher_capsula_retangular(retangulo, borda);
        let espessura = 1.25;
        let interno = Rect::new(
            retangulo.x + espessura,
            retangulo.y + espessura,
            (retangulo.w - espessura * 2.0).max(2.0),
            (retangulo.h - espessura * 2.0).max(2.0),
        );
        preencher_capsula_retangular(interno, cor_fundo);
    } else {
        preencher_capsula_retangular(retangulo, cor_fundo);
    }
}

struct ItemBarra<'a> {
    rotulo: String,
    cor_ponto: Option<Color>,
    ativo: bool,
    acao: AcaoInterface,
    marcador: &'a str,
}

pub fn desenhar_e_interagir_interface(
    interface: &mut EstadoInterface,
    mundo: &MundoSimulacao,
    tipografia: &Tipografia,
    cursor: Vec2,
    clique_esquerdo: bool,
) -> AcaoInterface {
    let mut acao_escolhida = AcaoInterface::Nenhuma;
    let largura_tela = screen_width();
    let altura_tela = screen_height();

    if !interface.barra_visivel {
        interface.altura_barra_atual = 0.0;
        if interface.mostrar_ajuda {
            desenhar_cartao_ajuda(tipografia, largura_tela, altura_tela);
            if clique_esquerdo {
                return AcaoInterface::AlternarAjuda;
            }
        }
        return acao_escolhida;
    }

    let mut itens_esquerda: Vec<ItemBarra> = Vec::with_capacity(10);
    for material in MaterialParticula::todos() {
        itens_esquerda.push(ItemBarra {
            rotulo: format!("{} {}", material.atalho(), material.nome()),
            cor_ponto: Some(material.cor_icone()),
            ativo: interface.ferramenta == FerramentaAtiva::Material(material),
            acao: AcaoInterface::SelecionarFerramenta(FerramentaAtiva::Material(material)),
            marcador: "",
        });
    }

    itens_esquerda.push(ItemBarra {
        rotulo: "7 Parede".to_string(),
        cor_ponto: None,
        ativo: interface.ferramenta == FerramentaAtiva::Parede,
        acao: AcaoInterface::SelecionarFerramenta(FerramentaAtiva::Parede),
        marcador: "|",
    });
    itens_esquerda.push(ItemBarra {
        rotulo: "8 Poço".to_string(),
        cor_ponto: Some(mundo.modo_forca.cor()),
        ativo: interface.ferramenta == FerramentaAtiva::Ancora,
        acao: AcaoInterface::SelecionarFerramenta(FerramentaAtiva::Ancora),
        marcador: "",
    });
    itens_esquerda.push(ItemBarra {
        rotulo: "9 Fonte".to_string(),
        cor_ponto: Some(interface.ultimo_material.cor_icone()),
        ativo: interface.ferramenta == FerramentaAtiva::Fonte,
        acao: AcaoInterface::SelecionarFerramenta(FerramentaAtiva::Fonte),
        marcador: "",
    });
    itens_esquerda.push(ItemBarra {
        rotulo: "0 Apagar".to_string(),
        cor_ponto: None,
        ativo: interface.ferramenta == FerramentaAtiva::Borracha,
        acao: AcaoInterface::SelecionarFerramenta(FerramentaAtiva::Borracha),
        marcador: "",
    });

    let itens_direita = [
        ItemBarra {
            rotulo: format!("F {}", mundo.modo_forca.nome()),
            cor_ponto: Some(mundo.modo_forca.cor()),
            ativo: false,
            acao: AcaoInterface::AlternarForca,
            marcador: "|",
        },
        ItemBarra {
            rotulo: format!("G {}", mundo.modo_gravidade.nome()),
            cor_ponto: None,
            ativo: mundo.modo_gravidade != crate::ambiente::ModoGravidade::Baixo,
            acao: AcaoInterface::AlternarGravidade,
            marcador: "",
        },
        ItemBarra {
            rotulo: format!("V {}", mundo.modo_visual.nome()),
            cor_ponto: None,
            ativo: mundo.modo_visual != ModoVisual::Material,
            acao: AcaoInterface::AlternarVisual,
            marcador: "",
        },
        ItemBarra {
            rotulo: format!("C {}", interface.cena_atual.nome()),
            cor_ponto: None,
            ativo: false,
            acao: AcaoInterface::ProximaCena,
            marcador: "",
        },
        ItemBarra {
            rotulo: "?".to_string(),
            cor_ponto: None,
            ativo: interface.mostrar_ajuda,
            acao: AcaoInterface::AlternarAjuda,
            marcador: "",
        },
    ];

    let tamanho_fonte: u16 = 14;
    let altura_pilula = 26.0;
    let espacamento = 5.0;
    let margem_lateral = 10.0;

    let quadros_por_segundo = get_fps();
    let texto_estado = if mundo.pausado {
        format!("{} · PAUSADO", mundo.particulas.len())
    } else {
        format!("{} · {} FPS", mundo.particulas.len(), quadros_por_segundo)
    };
    let dimensao_estado = tipografia.medir(&texto_estado, tamanho_fonte);

    let mut largura_total_estimada = margem_lateral * 2.0 + dimensao_estado.width + 24.0;
    for item in itens_esquerda.iter().chain(itens_direita.iter()) {
        let medicao = tipografia.medir(&item.rotulo, tamanho_fonte);
        let extra_ponto = if item.cor_ponto.is_some() { 11.0 } else { 0.0 };
        let extra_divisor = if !item.marcador.is_empty() { 10.0 } else { 0.0 };
        largura_total_estimada += medicao.width + 18.0 + extra_ponto + espacamento + extra_divisor;
    }

    let usar_duas_linhas = largura_total_estimada > largura_tela;
    let altura_barra = if usar_duas_linhas { 74.0 } else { 42.0 };
    interface.altura_barra_atual = altura_barra;

    draw_rectangle(0.0, 0.0, largura_tela, altura_barra, Color::new(0.055, 0.065, 0.095, 1.0));
    draw_line(
        0.0,
        altura_barra,
        largura_tela,
        altura_barra,
        1.0,
        Color::new(0.16, 0.20, 0.28, 1.0),
    );

    let mut cursor_x = margem_lateral;
    let mut cursor_y = 8.0;

    for item in itens_esquerda {
        if !item.marcador.is_empty() && cursor_x > margem_lateral + 4.0 {
            draw_line(
                cursor_x + 2.0,
                cursor_y + 5.0,
                cursor_x + 2.0,
                cursor_y + altura_pilula - 5.0,
                1.0,
                Color::new(0.22, 0.26, 0.36, 1.0),
            );
            cursor_x += 9.0;
        }

        let medicao = tipografia.medir(&item.rotulo, tamanho_fonte);
        let largura_ponto = if item.cor_ponto.is_some() { 11.0 } else { 0.0 };
        let largura_pilula = (medicao.width + 18.0 + largura_ponto).max(26.0);

        if cursor_x + largura_pilula > largura_tela - margem_lateral && cursor_x > margem_lateral {
            cursor_x = margem_lateral;
            cursor_y += altura_pilula + 6.0;
        }

        let retangulo = Rect::new(cursor_x, cursor_y, largura_pilula, altura_pilula);
        if renderizar_item_pilula(&retangulo, &item, tipografia, tamanho_fonte, cursor, clique_esquerdo) {
            acao_escolhida = item.acao;
        }
        cursor_x += largura_pilula + espacamento;
    }

    let mut largura_bloco_direito = dimensao_estado.width + 14.0;
    for item in &itens_direita {
        let medicao = tipografia.medir(&item.rotulo, tamanho_fonte);
        let largura_ponto = if item.cor_ponto.is_some() { 11.0 } else { 0.0 };
        largura_bloco_direito += (medicao.width + 18.0 + largura_ponto).max(26.0) + espacamento;
    }

    let (mut direita_x, direita_y) = if usar_duas_linhas {
        (margem_lateral, 8.0 + altura_pilula + 6.0)
    } else {
        (
            (largura_tela - margem_lateral - largura_bloco_direito).max(cursor_x + 8.0),
            8.0,
        )
    };

    let cor_estado = if mundo.pausado {
        Color::new(0.98, 0.76, 0.32, 1.0)
    } else {
        Color::new(0.56, 0.64, 0.76, 1.0)
    };
    let referencia_vertical = tipografia.medir("Ag", tamanho_fonte);
    let linha_base_direita =
        direita_y + (altura_pilula - referencia_vertical.height) * 0.5 + referencia_vertical.offset_y;
    tipografia.desenhar(
        &texto_estado,
        direita_x,
        linha_base_direita,
        tamanho_fonte,
        cor_estado,
    );
    direita_x += dimensao_estado.width + 12.0;

    for item in itens_direita {
        let medicao = tipografia.medir(&item.rotulo, tamanho_fonte);
        let largura_ponto = if item.cor_ponto.is_some() { 11.0 } else { 0.0 };
        let largura_pilula = (medicao.width + 18.0 + largura_ponto).max(26.0);
        let retangulo = Rect::new(direita_x, direita_y, largura_pilula, altura_pilula);
        if renderizar_item_pilula(&retangulo, &item, tipografia, tamanho_fonte, cursor, clique_esquerdo) {
            acao_escolhida = item.acao;
        }
        direita_x += largura_pilula + espacamento;
    }

    if interface.mostrar_ajuda {
        desenhar_cartao_ajuda(tipografia, largura_tela, altura_tela);
        if clique_esquerdo && matches!(acao_escolhida, AcaoInterface::Nenhuma) {
            acao_escolhida = AcaoInterface::AlternarAjuda;
        }
    }

    acao_escolhida
}

fn renderizar_item_pilula(
    retangulo: &Rect,
    item: &ItemBarra,
    tipografia: &Tipografia,
    tamanho_fonte: u16,
    cursor: Vec2,
    clique_esquerdo: bool,
) -> bool {
    let sobreposto = retangulo.contains(cursor);
    let cor_fundo = if item.ativo {
        Color::new(0.19, 0.25, 0.38, 1.0)
    } else if sobreposto {
        Color::new(0.14, 0.17, 0.25, 1.0)
    } else {
        Color::new(0.095, 0.115, 0.165, 1.0)
    };

    let cor_borda = if item.ativo {
        Some(item.cor_ponto.unwrap_or(Color::new(0.45, 0.65, 0.95, 1.0)))
    } else {
        None
    };

    desenhar_pilula_opaca(*retangulo, cor_fundo, cor_borda);

    let mut texto_x = retangulo.x + 9.0;
    if let Some(cor_ponto) = item.cor_ponto {
        draw_circle(texto_x + 3.0, retangulo.y + retangulo.h * 0.5, 3.6, cor_ponto);
        texto_x += 11.0;
    } else if item.rotulo == "?" {
        let medicao = tipografia.medir(&item.rotulo, tamanho_fonte);
        texto_x = retangulo.x + (retangulo.w - medicao.width) * 0.5;
    }

    let cor_texto = if item.ativo || sobreposto {
        Color::new(0.95, 0.97, 1.0, 1.0)
    } else {
        Color::new(0.72, 0.78, 0.88, 1.0)
    };

    let referencia = tipografia.medir("Ag", tamanho_fonte);
    let linha_base = retangulo.y + (retangulo.h - referencia.height) * 0.5 + referencia.offset_y;
    tipografia.desenhar(&item.rotulo, texto_x, linha_base, tamanho_fonte, cor_texto);

    sobreposto && clique_esquerdo
}

fn desenhar_cartao_ajuda(tipografia: &Tipografia, largura_tela: f32, altura_tela: f32) {
    draw_rectangle(0.0, 0.0, largura_tela, altura_tela, Color::new(0.02, 0.025, 0.04, 0.62));

    let largura_cartao = 470.0f32.min(largura_tela - 32.0);
    let altura_cartao = 316.0f32.min(altura_tela - 32.0);
    let origem_x = (largura_tela - largura_cartao) * 0.5;
    let origem_y = (altura_tela - altura_cartao) * 0.5;

    draw_rectangle(
        origem_x,
        origem_y,
        largura_cartao,
        altura_cartao,
        Color::new(0.07, 0.085, 0.125, 1.0),
    );
    draw_rectangle_lines(
        origem_x,
        origem_y,
        largura_cartao,
        altura_cartao,
        1.5,
        Color::new(0.24, 0.30, 0.44, 1.0),
    );

    tipografia.desenhar(
        "Atalhos e Controles",
        origem_x + 22.0,
        origem_y + 32.0,
        18,
        Color::new(0.94, 0.96, 1.0, 1.0),
    );

    let atalhos = [
        ("Botão Esquerdo", "Usar ferramenta ou criar partículas"),
        ("Botão Direito", "Aplicar força contínua no cursor"),
        ("Roda do Mouse", "Ajustar raio do pincel e alcance"),
        ("Teclas 1 a 6", "Materiais (Elástico, Água, Areia, Gás, Fogo, Ímã)"),
        ("Teclas 7 / 8 / 9 / 0", "Parede (Shift alinha), Poço, Fonte, Apagar"),
        ("F / G / V / C", "Alternar Força, Gravidade, Visual e Cena"),
        ("Tecla Q", "Disparar onda de choque radial"),
        ("Espaço / Ponto (.)", "Pausar simulação / Avançar um quadro"),
        ("R / X / Tab / H", "Reiniciar cena / Limpar tudo / Barra / Ajuda"),
    ];

    let mut linha_y = origem_y + 64.0;
    for (tecla, descricao) in atalhos {
        tipografia.desenhar(
            tecla,
            origem_x + 22.0,
            linha_y,
            14,
            Color::new(0.48, 0.76, 0.98, 1.0),
        );
        tipografia.desenhar(
            descricao,
            origem_x + 178.0,
            linha_y,
            14,
            Color::new(0.78, 0.83, 0.92, 1.0),
        );
        linha_y += 26.0;
    }
}
