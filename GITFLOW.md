# GitFlow Workflow

Este proyecto sigue el modelo **GitFlow** para gestionar ramas y releases.

## Estructura de Ramas

### Ramas Permanentes

- **`main`** - Código en producción. Solo se mergea desde `release/*`
- **`develop`** - Rama de integración para desarrollo

### Ramas Temporales

- **`feature/*`** - Nuevas funcionalidades
- **`release/*`** - Preparación de releases
- **`hotfix/*`** - Fixes urgentes a producción

---

## Flujo de Trabajo

### 1. Desarrollo de Features

```bash
# Crear feature branch desde develop
git checkout develop
git pull origin develop
git checkout -b feature/nombre-feature

# Desarrollar y commitear
git add .
git commit -m "feat: descripción del cambio"

# Push y crear PR a develop
git push origin feature/nombre-feature
```

**PR**: `feature/nombre-feature` → `develop`
- ✅ Debe pasar CI checks (tests, clippy, coverage)
- ✅ Code review requerido
- ✅ Merge squash o merge commit según preferencia

---

### 2. Preparar Release

```bash
# Crear release branch desde develop
git checkout develop
git pull origin develop
git checkout -b release/1.2.3

# Actualizar versión en Cargo.toml
vim Cargo.toml  # version = "1.2.3"

# Actualizar CHANGELOG.md
vim CHANGELOG.md

# Commit cambios de versión
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump version to 1.2.3"

# Push release branch
git push origin release/1.2.3
```

**PR**: `release/1.2.3` → `main`
- ✅ Debe pasar CI checks
- ✅ Revisión obligatoria
- **Al mergear**:
  - 🤖 CI crea tag `v1.2.3` automáticamente
  - 🤖 CI genera GitHub Release
  - 🤖 CI publica a crates.io (si `CARGO_REGISTRY_TOKEN` existe)
  - 🤖 CI compila binarios para Linux, macOS, Windows

**Después del merge a main**:

```bash
# Hacer merge de release a develop para sincronizar
git checkout develop
git pull origin develop
git merge release/1.2.3
git push origin develop

# Borrar release branch
git branch -d release/1.2.3
git push origin --delete release/1.2.3
```

---

### 3. Hotfixes

```bash
# Crear hotfix branch desde main
git checkout main
git pull origin main
git checkout -b hotfix/1.2.4

# Fix crítico
git add .
git commit -m "fix: descripción del hotfix"

# Actualizar versión
vim Cargo.toml  # version = "1.2.4"
git add Cargo.toml
git commit -m "chore(release): bump version to 1.2.4"

# Push
git push origin hotfix/1.2.4
```

**PR**: `hotfix/1.2.4` → `main`
- Mismo proceso que `release/*`
- Al mergear se crea tag automáticamente

**Después del merge**:

```bash
# Mergear a develop también
git checkout develop
git pull origin develop
git merge hotfix/1.2.4
git push origin develop

# Borrar hotfix branch
git branch -d hotfix/1.2.4
git push origin --delete hotfix/1.2.4
```

---

## CI/CD Pipelines

### CI Checks (en todos los PRs)

`.github/workflows/ci.yml` corre en PRs a `develop` y `main`:

- ✅ `cargo test` - Suite de tests completa
- ✅ `cargo clippy` - Linting
- ✅ `cargo tarpaulin` - Coverage mínimo 60%
- ✅ `cargo audit` - Security vulnerabilities
- ✅ Multi-OS: Ubuntu, macOS

### Release Automation

`.github/workflows/release-from-branch.yml` se dispara al mergear PR de `release/*` o `hotfix/*` a `main`:

1. **Extrae versión** del nombre del branch (`release/1.2.3` → `1.2.3`)
2. **Valida** que Cargo.toml tenga la misma versión
3. **Crea tag** `v1.2.3`
4. **Genera GitHub Release** con changelog
5. **Publica a crates.io** (si token configurado)
6. **Compila binarios** para Linux, macOS, Windows

---

## Branch Protection Rules

**Para `main`**:
- ✅ Require pull request reviews before merging
- ✅ Require status checks to pass (CI)
- ✅ Require branches to be up to date
- ✅ Restrict who can push to matching branches

**Para `develop`**:
- ✅ Require pull request reviews before merging
- ✅ Require status checks to pass (CI)
- ✅ Require branches to be up to date

---

## Configurar Secrets

Para publicar automáticamente a crates.io:

1. Obtener token: https://crates.io/me
2. En GitHub: Settings → Secrets → Actions → New repository secret
3. Nombre: `CARGO_REGISTRY_TOKEN`
4. Valor: tu token de crates.io

---

## Ejemplo de CHANGELOG.md

```markdown
# Changelog

## [1.2.3] - 2026-04-01

### Added
- Semantic EdgeKind inference from LSP hover
- DOT and Mermaid output formats
- Comprehensive error templates

### Changed
- Improved cache invalidation (selective by file)

### Fixed
- Public exports resolution for library crates

## [1.2.2] - 2026-03-25
...
```

---

## Comandos Útiles

```bash
# Ver ramas
git branch -a

# Ver tags
git tag -l

# Borrar rama local
git branch -d feature/nombre

# Borrar rama remota
git push origin --delete feature/nombre

# Ver último tag
git describe --tags --abbrev=0

# Crear release manualmente (fallback)
git tag -a v1.2.3 -m "Release 1.2.3"
git push origin v1.2.3
```

---

## Troubleshooting

**CI no crea el tag**:
- Verificar que el branch se llame exactamente `release/x.y.z`
- Verificar que Cargo.toml tenga version = "x.y.z"
- Revisar logs en Actions tab

**CI no publica a crates.io**:
- Verificar que `CARGO_REGISTRY_TOKEN` esté configurado
- Verificar que la versión no esté ya publicada
- El workflow tiene `continue-on-error: true` para este step

**Tests fallan en CI pero pasan local**:
- Cache de cargo: borrar cache en GitHub Actions settings
- Versiones de Rust diferentes: CI usa `stable` channel

---

*Última actualización: 2026-04-01*
