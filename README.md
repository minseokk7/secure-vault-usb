# 🔒 SecureVault
### USB Portable Encrypted Safe - Your Data, Only for Your Eyes.

![Version](https://img.shields.io/badge/version-v1.1.3-blue) ![Security](https://img.shields.io/badge/security-AES256--GCM-green) ![Platform](https://img.shields.io/badge/platform-Windows-blue) ![License](https://img.shields.io/badge/license-MIT-lightgrey)

**SecureVault**는 USB 드라이브에 최적화된 **완전 오프라인 보안 파일 금고**입니다.
클라우드를 믿지 못하시나요? 당신의 가장 소중한 데이터를 물리적으로 격리된 USB에 보관하고, 어디서든 안전하게 접근하세요.

---

## 🛡️ 핵심 보안 기능 (Key Features)

### 1. 🔌 USB Kill Switch (자동 긴급 종료)
SecureVault는 USB 드라이브에서 실행되도록 설계되었습니다.
- **USB 제거 감지**: 앱 실행 중 USB가 물리적으로 분리되면, **1초 이내에 모든 메모리를 소거하고 앱을 강제 종료**합니다.
- 흔적 없는 종료: 운영체제에 실행 흔적을 최소화합니다.

### 2. 💣 듀레스 패스워드 (Duress mode)
누군가에게 협박을 당해 비밀번호 입력을 강요받고 있나요?
- **가짜 PIN 설정**: 미리 설정해둔 '듀레스 PIN'을 입력하세요.
- **Fake Vault**: 앱이 정상적으로 열리지만, **모든 데이터가 텅 빈 상태**로 표시됩니다. 공격자는 데이터가 없다고 믿게 됩니다.
- (향후 업데이트) Panic Mode: 듀레스 PIN 입력 시 데이터를 영구 파괴하는 옵션 제공 예정.

### 3. 🔐 군사 등급 암호화 (Military-Grade Encryption)
모든 데이터는 기기(USB)에 저장되기 전에 암호화됩니다.
- **Master Key**: Argon2id KDF(Key Derivation Function)를 사용하여 PIN으로부터 강력한 마스터 키 생성.
- **File Encryption**: AES-256-GCM 알고리즘으로 파일 내용 암호화.
- **Metadata Encryption**: 파일명, 폴더 구조, 메모 내용 등 모든 메타데이터도 SQLCipher(AES-256)로 암호화된 DB에 저장됩니다.
- **Zero Knowledge**: 오직 당신의 PIN만이 데이터를 복호화할 수 있습니다. 개발자도 복구할 수 없습니다.
- **Zeroize Memory**: 민감한 키와 암호 데이터는 사용 후 메모리에서 안전하게 소거됩니다 (Rust `zeroize` 크레이트 사용).

### 4. �️ 안전한 파일 삭제 (Secure Delete)
- **3회 덮어쓰기**: 파일 삭제 시 디스크의 암호화 파일을 0으로 3회 덮어쓴 후 삭제합니다.
- **복구 불가능**: 포렌식 도구로도 삭제된 파일을 복구할 수 없습니다.

### 5. �📝 보안 메모장 & 뷰어
- **Encrypted Memo**: 텍스트 메모를 암호화하여 DB에 안전하게 저장합니다.
- **In-Memory Viewer**: 이미지, 텍스트 파일을 디스크에 임시 파일로 저장하지 않고, **RAM에서 즉시 복호화하여 표시**합니다. 뷰어를 닫으면 데이터는 메모리에서 즉시 사라집니다.

### 6. 🚫 100% 오프라인 & 네트워크 차단
- SecureVault는 인터넷 연결을 전혀 사용하지 않습니다.
- 백엔드(Rust)에서 네트워크 관련 라이브러리를 의도적으로 배제하거나 차단하여, 해킹이나 데이터 유출 가능성을 원천 봉쇄했습니다.

---

## 📥 설치 및 실행 방법

### 요구 사항
- 운영체제: **Windows 10/11 (64-bit)**
- USB 드라이브 (권장: USB 3.0 이상 고속 메모리)

### 설치 (USB)
1. [최신 릴리스](https://github.com/minseokk7/secure-vault-usb/releases)에서 `SecureVault_USB_Installer.exe`를 다운로드합니다.
2. PC에 USB 드라이브를 연결합니다.
3. 인스톨러를 실행하고, 설치 위치로 **USB 드라이브**를 선택합니다.

### 실행
1. USB 드라이브의 `SecureVault` 폴더로 이동합니다.
2. `SecureVault.exe`를 실행합니다.
3. 최초 실행 시 **PIN 번호(4~12자리)**를 설정합니다. 이 PIN은 절대 잊어버리면 안 됩니다!
4. (선택) 설정 메뉴에서 **듀레스 PIN**을 추가로 설정하여 비상 상황에 대비하세요.

---

## 🛠️ 기술 스택 (Tech Stack)

이 프로젝트는 최신 기술을 사용하여 보안과 성능을 모두 잡았습니다.

- **Frontend**: SvelteKit, TypeScript, TailwindCSS
- **Backend (App Logic)**: Rust, Tauri v2
- **Database**: SQLite (+ SQLCipher for encryption)
- **Encryption**: `ring`, `aes-gcm`, `argon2`, `zeroize` (Rust crates)

---

## ⚠️ 주의사항

- **비밀번호 분실 시 복구 불가능**: 마스터 키는 암호화되어 저장되며, 오직 PIN으로만 풀 수 있습니다. PIN을 분실하면 모든 데이터를 잃게 됩니다. 복구 키 코드를 반드시 안전한 곳에 보관하세요.
- **데이터 백업**: USB 메모리는 분실이나 고장 위험이 있습니다. 중요한 데이터는 주기적으로 다중 백업하시기 바랍니다.

---

## 📄 라이선스
MIT License
