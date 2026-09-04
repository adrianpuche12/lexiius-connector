# Code Signing Policy

Este proyecto firma y distribuye artefactos de release (instaladores de
Windows `.exe`/`.msi`) generados por su pipeline de CI en GitHub Actions
(`.github/workflows/build.yml`), a partir del código fuente público de este
mismo repositorio.

## Roles

Lexiius Connector es mantenido por una sola persona, que cumple los tres
roles requeridos:

| Rol | Responsable | Alcance |
|---|---|---|
| Author (acceso de commit) | Adrian Pucheta ([@adrianpuche12](https://github.com/adrianpuche12)) | Único colaborador con permiso de escritura sobre `main`/`master`. |
| Reviewer (revisión de PRs externas) | Adrian Pucheta | Toda contribución externa se revisa manualmente antes de mergear. |
| Approver (autoriza cada firma) | Adrian Pucheta | Autoriza manualmente cada solicitud de firma antes de publicar un release. |

La cuenta de GitHub usada para autenticar el pipeline de CI tiene
autenticación de dos factores (2FA) habilitada.

## Qué se firma

Únicamente los binarios de Windows generados por el propio pipeline de CI de
este repositorio a partir de su código fuente público (job `build-windows`
en `.github/workflows/build.yml`). No se firman binarios de terceros ni
artefactos que no provengan de este repositorio.

## Compromiso

- Este proyecto no distribuye malware ni "potentially unwanted programs".
- No firma herramientas de explotación de vulnerabilidades ni hacking tools.
- Respeta la privacidad del usuario: la aplicación solo se comunica con
  `app.lexiius.com` y con la instalación local de Claude Desktop del propio
  usuario.
- El proyecto se mantiene activamente (ver historial de commits y releases).

## Contacto

Para reportar un problema de seguridad relacionado con la firma de este
proyecto: TODO-completar-email-de-contacto-publico
