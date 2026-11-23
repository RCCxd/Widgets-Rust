# Widgets

Widgets de área de trabalho para Windows com visual **Liquid Glass**: relógio, calendário com compromissos e um player de música que mostra o que está tocando no **Spotify**, **YouTube** e outros apps compatíveis. Feito com Rust + Tauri, leve (poucos MB e baixo uso de memória).

## Recursos

- **Player de música** compacto que expande ao clicar: capa, barra de progresso clicável, anterior, play/pause e próxima.
- **Relógio** com formato 12/24 h e segundos opcionais.
- **Calendário** com feriados nacionais, navegação entre meses e compromissos locais.
- **Central de configurações** com prévia em tempo real: tema, cor de destaque, transparência, arredondamento e escala.
- Cada widget é uma janela independente: arraste para onde quiser.
- Opção de **iniciar com o Windows**.

## Baixar e usar

1. Abra a aba **Releases** deste repositório e baixe o `widgets.exe`.
2. Dê dois cliques para abrir.
3. O Windows pode mostrar o aviso *"O Windows protegeu seu computador"*, porque o programa não é assinado. Clique em **Mais informações → Executar assim mesmo**.
4. Clique no ícone azul da bandeja (perto do relógio do Windows) para abrir as **Configurações** ou **Sair**.

Requisitos: Windows 10/11 com o WebView2 (já vem no Windows 11).

## Compilar a partir do código

1. Instale o [Rust](https://rustup.rs) (aceite instalar o *Visual Studio C++ Build Tools*).
2. Rode:
   ```
   cd src-tauri
   cargo run --release
   ```
   ou dê dois cliques no `iniciar.bat`.

O executável fica em `src-tauri/target/release/widgets.exe`.

## Dados salvos

As configurações e os compromissos ficam em `%APPDATA%\dev.renan.widgets\` (`settings.json` e `events.json`). Nada é enviado para a internet.

## Como o player funciona

O app lê o *Global System Media Transport Controls* do Windows, o mesmo que alimenta os controles de mídia do sistema. Por isso funciona com qualquer app que apareça ali, sem login e sem API.

## Estrutura

```
src-tauri/   backend em Rust (mídia do Windows, configurações, bandeja)
ui/          interface (HTML/CSS/JS) dos widgets e das configurações
.github/     automação que compila o .exe a cada versão publicada
```
