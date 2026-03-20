# Madome

## What This Is

외부 소스(hitomi.la)로부터 만화 작품의 메타데이터와 이미지를 미러링하는 서비스. Rust 기반 마이크로서비스 아키텍처로 구성되며, 소규모 커뮤니티를 대상으로 한다. 추후 이미지 검열 자동화를 구현하여 오픈 서비스로 확장 가능성이 있다.

## Core Value

외부 소스의 작품을 안정적으로 미러링하고, 인증된 사용자가 작품을 열람할 수 있어야 한다.

## Architecture

### Service Topology

Cargo workspace monorepo로 구성된 6개 바이너리:

| Binary | Role | Communication |
|--------|------|---------------|
| **gateway** | REST 진입점, JWT 검증, gRPC 라우팅 | REST (외부) → gRPC (내부) |
| **auth** | JWT 발급, 세션 관리, Passkey | gRPC |
| **catalog** | 작품 CRUD, 태그 쿼리, 퍼블리싱 | gRPC |
| **user** | tastes (like/dislike), histories | gRPC |
| **file** | 이미지 업로드/저장 전담 | REST (nginx 리버스 프록시) |
| **scraper** | 외부 소스 미러링 (별도 서버) | REST → Gateway (API Key) |

### Communication Flow

```
Client → REST → nginx → Gateway → gRPC → { auth, catalog, user }
Scraper → REST (API Key) → nginx → Gateway → gRPC → catalog
Scraper → REST → nginx → File Service → Local Filesystem
Image Read: Client → nginx auth_request → Gateway (인증) → nginx serves file
```

- **내부 서비스 간**: gRPC (tonic)
- **외부 API**: REST (axum, JSON)
- **Scraper → Gateway**: REST + API Key 인증
- **이미지 서빙**: nginx `auth_request` + `auth_request_set`으로 쿠키 갱신 전달
- 모든 서비스는 nginx 리버스 프록시 뒤에서 실행

### Auth Design

**Passkey + JWT + Session 하이브리드:**

1. Passkey(webauthn-rs)로 인증 → 세션 생성 + JWT 발급
2. JWT access token (15분 TTL), HttpOnly Cookie로 관리
3. 클라이언트는 토큰 갱신을 신경쓰지 않음 (서버 측 자동 관리)

**Gateway JWT 처리 흐름:**

1. JWT 유효 → stateless 통과 (auth 서비스 호출 없음)
2. JWT 만료 + Grace Period 이내 → 통과 + 새 JWT 쿠키 세팅
3. JWT 만료 + Grace Period 초과 → 세션 확인 후 새 JWT 발급
4. 세션 무효 → 요청 거절

**중복 JWT 발급 방지:** 세션별 최근 발급 JWT 캐싱 (N초 이내 동일 JWT 재사용)

**이미지 요청 시 토큰 갱신:** nginx `auth_request_set $auth_cookie $upstream_http_set_cookie` + `add_header Set-Cookie $auth_cookie`로 auth 서브요청의 Set-Cookie를 클라이언트에 전달

### Upload Scenario

1. Scraper가 주기적으로 외부 소스에서 새 작품 확인
2. 새 작품 발견 → Catalog에 메타데이터 업로드 (unpublished, 작품 정보+태그 해시값 포함)
3. 이미지를 File 서비스에 업로드
4. Scraper가 Catalog에 publish 요청
5. Catalog에서 데이터의 페이지 수와 실제 이미지 수 검증 후 publish

### Update/Renewal Scenario

**업데이트 확인 빈도 (단계적 감소):**
- 6시간 이내: 5분마다
- 6h~24h: 30분마다
- 1d~7d: 2시간마다
- 7d+: 12시간마다

**작품 ID 동일:** Scraper가 Catalog에 정보 업데이트 요청. 끝.

**작품 ID 변경 (갱신):**
- 기존 작품 데이터 보존 (덮어쓰기 금지)
- 중복 작품 간 graph/relation으로 연결
- Canonical ID: 새 ID가 canonical, old ID는 리다이렉트
- 작품 정보 페이지에서 갱신 이력 조회 가능
- DB 기반 queue로 타 서비스(user 등) 작품 ID 참조 업데이트
- 이미지 파일은 이동하지 않음
- 추후 메시지 브로커로 마이그레이션 가능

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Gateway REST API + gRPC 라우팅
- [ ] Passkey 인증 (webauthn-rs)
- [ ] JWT + Session 하이브리드 인증
- [ ] Catalog 작품 CRUD (publish workflow 포함)
- [ ] 특정 태그를 포함하는 작품 조회 API
- [ ] 특정 태그들(복수)을 포함하는 작품 조회 API
- [ ] 특정 ID 목록으로 작품 조회 API
- [ ] 이미지 업로드 (File 서비스)
- [ ] nginx auth_request 기반 이미지 서빙
- [ ] 작품 정보 업데이트 확인 (단계적 빈도)
- [ ] 작품 ID 갱신 처리 (graph relation, canonical ID, queue)
- [ ] User tastes (like/dislike)
- [ ] User histories
- [ ] Scraper: hitomi.la 새 작품 감지 및 미러링

### Out of Scope

- Frontend UI — API 우선 구축, 프론트엔드는 나중에
- 이미지 검열 자동화 — 오픈 서비스 전환 시 고려
- 신고/모더레이션 — v2+
- 메시지 브로커 (RabbitMQ 등) — DB 기반 queue로 충분, 추후 검토
- OAuth/소셜 로그인 — Passkey만 지원
- 모바일 앱 — 웹 API 우선

## Context

- hitomi.la의 작품 번호가 클수록 최근 작품
- 작품 갱신은 주로 외부 소스의 중복 제거로 발생
- 이미 로컬 파일시스템에 기존 이미지/데이터 존재, 릴리즈 직전 마이그레이션 예정
- 별도 서비스의 DB 간 직접 FK 불가 → 간접적 FK로 취급 (queue 기반 동기화)
- `hitomi_la` crate 사용 가능

## Constraints

- **Tech Stack**: Rust (Cargo workspace monorepo)
- **Database**: PostgreSQL (sea-orm)
- **Internal Comm**: gRPC (tonic + prost)
- **External API**: REST (axum)
- **Image Storage**: 로컬 파일시스템
- **Reverse Proxy**: nginx (auth_request, 리버스 프록시)
- **Scraper Deployment**: API 서버와 별도 서버에서 실행
- **Auth**: Passkey only (webauthn-rs, minicbor for AAGUID)
- **Token**: JWT (jsonwebtoken, aws_lc backend)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Microservice (6 binaries) | 서비스별 독립 배포/스케일링, 관심사 분리 | — Pending |
| gRPC 내부 통신 | 타입 안전성, 성능, 코드 생성 | — Pending |
| API Gateway 패턴 | JWT 검증 중앙화, 서비스 직접 노출 방지 | — Pending |
| nginx auth_request + auth_request_set | 이미지 서빙 인증 + 쿠키 갱신 전달 | — Pending |
| DB 기반 queue (not 메시지 브로커) | 인프라 복잡도 최소화, 추후 마이그레이션 가능 | — Pending |
| Canonical ID + graph relation | 기존 데이터 보존, 갱신 이력 추적 | — Pending |
| JWT 중복 발급 방지 (세션별 캐싱) | 동시 요청 시 불필요한 토큰 생성 방지 | — Pending |
| File 서비스 별도 분리 | Gateway 부담 방지, 이미지 트래픽 격리 | — Pending |
| 업데이트 확인 빈도 단계적 감소 | 최신 작품일수록 변경 가능성 높음 | — Pending |

---
*Last updated: 2026-03-21 after initialization*
