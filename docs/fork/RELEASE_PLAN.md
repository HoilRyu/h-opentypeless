# Apple Silicon 시험판 준비 계획

2026-09-11. 릴리즈 후보 0.1.45-beta.3.
배포 소스는 `feat/h-foundation-direct-providers`의 검증된 깨끗한 커밋으로 고정한다.

## 범위

macOS 14 이상 Apple Silicon(arm64)용 자체 서명 DMG 한 개를 배포한다.
Intel Mac, Android, Windows, Linux 및 Universal 패키지는 이번 배포 범위에 포함하지 않는다.
MLX 실행 환경의 최소 OS에 맞춰 앱 최소 OS도 14.0으로 설정한다.
M 시리즈 세대별 파일을 나누지 않는다. 모델 가중치는 포함하지 않는다.

## 순서와 완료 기준

1. 버전·Mac 단독 패키지 검사·릴리즈 노트·빌드 복사 제외 규칙을 정리한다.
2. Rust/UI/Python 및 빌드·배포 설정 검사를 수행하고 소스를 커밋한다.
3. 고정 인증서와 credential helper를 재사용하여 release 앱과 DMG를 만든다.
4. DMG 무결성·번들 서명·CPU·버전·소스 SHA·체크섬을 확인한다.
5. 실제 배포 DMG의 신규 설치/업데이트 및 음성 입력·Fn·기록 재다듬기·음량 복구를 검증한다.
6. 패키지 폴더 VALIDATION.md에 자동 검사와 실제 장치 검사를 구분한다.
7. 검증한 커밋에 태그를 붙여 H 저장소로 푸시하고 Draft Pre-release를 준비한다.
8. 다운로드한 파일의 체크섬과 설치 결과까지 확인한 뒤 공개한다.

실제 실행 절차는 [RELEASING.md](RELEASING.md), 배포 설명은 [RELEASE_NOTES.md](RELEASE_NOTES.md)를 따른다.
미검증 항목을 통과로 기재하지 않는다. 이번 준비 작업에서 태그 생성이나 공개를 자동 수행하지 않는다.
