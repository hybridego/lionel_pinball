# WASM 빌드 및 실행 가이드

이 프로젝트는 [Trunk](https://trunkrs.dev/)를 사용하여 WebAssembly(WASM)로 빌드하고 실행할 수 있도록 이미 구성되어 있습니다.

## 1. 필수 도구 설치

먼저 Rust의 WASM 타겟과 Trunk 도구를 설치해야 합니다. 터미널에서 다음 명령어들을 순서대로 실행하세요.

```bash
# 1. WASM 타겟 추가
rustup target add wasm32-unknown-unknown

# 2. Trunk 도구 설치 (시간이 조금 걸릴 수 있습니다)
cargo install --locked trunk
```

## 2. 개발 서버 실행 (로컬 테스트)

개발 중에는 `trunk serve` 명령어를 사용하여 로컬 웹 서버를 띄우고 실시간으로 변경 사항을 확인할 수 있습니다.

```bash
trunk serve
```

- 실행 후 터미널에 표시되는 주소(보통 `http://127.0.0.1:8080`)를 브라우저에서 열면 게임을 플레이할 수 있습니다.
- 소스 코드를 수정하면 자동으로 다시 빌드되고 브라우저가 새로고침됩니다.

## 3. 배포용 빌드

최종 배포를 위한 파일을 생성하려면 `trunk build --release`를 사용합니다.

```bash
trunk build --release
```

- 빌드가 완료되면 `dist` 폴더에 `index.html`, `.wasm`, `.js` 등 배포에 필요한 모든 파일이 생성됩니다.
- 이 `dist` 폴더의 내용물을 GitHub Pages나 Vercel, Netlify 같은 정적 웹 호스팅 서비스에 업로드하면 됩니다.

## 문제 해결

만약 빌드 중 에러가 발생하면 다음을 확인해보세요:
- `Cargo.toml`에 `wasm-bindgen` 의존성이 있는지 확인 (현재 포함되어 있음)
- `rapier2d`가 `wasm-bindgen` 기능을 활성화했는지 확인 (현재 포함되어 있음)
