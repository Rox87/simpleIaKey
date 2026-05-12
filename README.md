# jKey (iaKeyLite)

<p align="center">
  <img src="assets/logo.png" alt="jKey Logo" width="200"/>
</p>

Uma ferramenta poderosa e leve escrita em Rust para integrar Inteligência Artificial (Gemini e DeepSeek) diretamente no seu fluxo de trabalho através de atalhos de teclado globais.

## 🚀 Funcionalidades

- **Melhoria Automática de Texto (F3)**: Selecione um texto, pressione F3 e receba uma versão aprimorada colada automaticamente.
- **Consulta Geral (F8)**: Transforme qualquer seleção em uma pergunta para a IA.
- **Geração de Código Python (F9)**: Gere trechos de código Python baseados no contexto selecionado.
- **Geração de HTML/Web (F10)**: Crie estruturas HTML rapidamente com IA.
- **Suporte Multi-Provedor**: Utilize Google Gemini e DeepSeek simultaneamente.
- **Instância Única Inteligente**: Se você tentar abrir o app novamente, a versão anterior é fechada e reiniciada automaticamente.
- **Modo Silencioso**: Funciona na bandeja do sistema (System Tray) consumindo o mínimo de recursos.

## 🛠️ Configuração

Para utilizar o jKey, você precisa de chaves de API dos provedores suportados.

1. Clone o repositório:
   ```bash
   git clone <repository-url>
   cd iaKeyLite
   ```

2. Crie um arquivo `.env` na raiz do projeto seguindo o modelo:
   ```env
   GEMINI_API_KEY=sua_chave_gemini_aqui
   DEEPSEEK_API_KEY=sua_chave_deepseek_aqui
   
   # Configurações de Modelos (Opcional)
   MODELO_GERAL=gemini-1.5-flash
   MODELO_CODIGO=deepseek-coder
   ```

## ⌨️ Atalhos de Teclado

| Tecla | Ação |
| :--- | :--- |
| **F3** | Melhora o texto selecionado (re-escrita inteligente) |
| **F8** | Envia o texto selecionado como uma pergunta geral |
| **F9** | Gera código Python baseado no texto |
| **F10** | Gera código HTML/Frontend baseado no texto |

## 📦 Como Rodar

### Requisitos
- [Rust Toolchain](https://rustup.rs/) (Windows recomendado)

### Execução em Desenvolvimento
```bash
cargo run
```

### Compilação de Produção (Otimizada)
```bash
cargo build --release
```
O executável será gerado em `target/release/jKey.exe`.

## 🎨 Design e Estética
O jKey foi projetado para ser "invisível" até que você precise dele. Com um ícone moderno na bandeja do sistema e respostas rápidas, ele se integra perfeitamente ao Windows.

## 📄 Licença
Este projeto é distribuído sob a licença MIT. Veja o arquivo `LICENSE` para mais detalhes.

---
<p align="center">Desenvolvido com ❤️ usando Rust</p>