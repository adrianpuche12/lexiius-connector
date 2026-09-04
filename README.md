# Lexiius Connector

Aplicación de escritorio liviana para Windows (Tauri v2 / Rust + React) que conecta
[Lexiius](https://app.lexiius.com) con Claude Desktop en un click: configura
automáticamente el servidor MCP local que le permite a Claude Desktop leer los
documentos anonimizados del vault del usuario, sin pasos manuales.

Este repositorio contiene **solo el connector** — el cliente ligero que corre en
la máquina del usuario. No incluye el backend de Lexiius, el motor de
anonimización de PII, ni ninguna lógica de negocio del producto; esos
componentes son privados.

## Qué hace

- Corre un servidor HTTP local (`127.0.0.1:47821`) que expone el vault
  anonimizado del usuario vía MCP (Model Context Protocol).
- Escribe la configuración necesaria en el archivo de config de Claude Desktop
  para registrar el servidor MCP, evitando que el usuario tenga que editarlo
  a mano.
- Se ejecuta en segundo plano con un ícono en la bandeja del sistema
  (`tauri-plugin-autostart`).

## Descargas

Los instaladores firmados se publican en
[GitHub Releases](https://github.com/adrianpuche12/lexiius-connector/releases).

## Build local

Requisitos: Node.js 20+, Rust estable, dependencias de sistema de Tauri v2
para Windows.

```bash
npm install
npm run tauri build
```

El pipeline de CI (`.github/workflows/build.yml`) compila y empaqueta el
instalador de Windows (NSIS + MSI) de forma reproducible a partir de este
mismo código fuente en cada push a `main`/`master` y en cada tag `v*`.

## Tests

```bash
npm test          # tests JS (Jest)
cd src-tauri && cargo test   # tests Rust
```

## Firma de código

Ver [SIGNING-POLICY.md](./SIGNING-POLICY.md).

## Licencia

[MIT](./LICENSE) — Copyright (c) 2026 Adrian Pucheta.
