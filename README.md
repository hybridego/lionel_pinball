# 🎮 Pinball Gacha Project

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/wasm-%235C4EE5.svg?style=for-the-badge&logo=webassembly&logoColor=white)
![GitHub Actions](https://img.shields.io/badge/github%20actions-%232671E5.svg?style=for-the-badge&logo=githubactions&logoColor=white)

> Rust와 WebAssembly로 제작된 현대적인 물리 기반 핀볼 게임으로, 역동적인 장애물과 가챠 시스템이 특징입니다.

## ✨ 주요 기능

- **🎯 물리 기반 게임플레이**: `rapier2d` 엔진을 사용하여 현실적인 충돌 및 반발 물리를 구현했습니다.
- **🌀 동적 장애물**:
  - **풍차 (Windmills)**: 다양한 속도와 색상을 가진 회전하는 장애물입니다.
  - **범퍼 (Bumpers)**: 충돌 시 네온 시각 효과와 함께 반응하는 탄성 핀입니다.
  - **이벤트 스포너 (Event Spawners)**: 화려한 색상의 기하학적 모양이 무작위로 생성됩니다.
- **🎲 가챠 시스템**: 먼저 들어온 순서 또는 늦게 들어온 순서를 결정 할 수 있습니다.
- **⚡ 고성능**: Rust로 제작되고 WebAssembly로 컴파일되어 브라우저에서 네이티브급 성능을 발휘합니다.
- **🎨 반응형 UI**: `egui`를 사용하여 구축된 깔끔한 인터페이스를 제공합니다.

## 🛠️ 기술 스택

- **언어**: [Rust](https://www.rust-lang.org/)
- **프론트엔드 프레임워크**: [egui](https://github.com/emilk/egui) (Immediate Mode GUI)
- **물리 엔진**: [Rapier2d](https://rapier.rs/)
- **빌드 도구**: [Trunk](https://trunkrs.dev/)
- **타겟**: WebAssembly (`wasm32-unknown-unknown`)

## 🚀 시작하기

### 필수 조건

Rust와 WASM 타겟이 설치되어 있어야 합니다:

```bash
# 1. WASM 타겟 추가
rustup target add wasm32-unknown-unknown

# 2. Trunk 도구 설치 (WASM 번들러)
cargo install --locked trunk
```

### 로컬 실행

핫 리로딩(Hot-reloading)을 지원하는 개발 서버를 시작합니다:

```bash
trunk serve
```

브라우저에서 `http://127.0.0.1:8080`을 열어 게임을 즐길 수 있습니다!

### 배포용 빌드

최적화된 릴리즈 버전을 빌드합니다:

```bash
trunk build --release
```

빌드가 완료되면 `dist` 디렉토리에 결과물이 생성되며, 이를 GitHub Pages, Vercel 등 정적 웹 호스팅 서비스에 바로 배포할 수 있습니다.

## 📦 배포

이 프로젝트는 **GitHub Actions**를 사용하여 자동 배포를 수행합니다. `main` 브랜치에 푸시하면 자동으로 빌드되어 GitHub Pages에 배포됩니다.

[lionel pinball](https://hybridego.github.io/lionel_pinball/)  

---

Made with ❤️ using Rust
