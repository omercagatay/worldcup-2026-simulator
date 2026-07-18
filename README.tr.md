# Dünya Kupası 2026 Simülatörü

[English](README.md) | [Türkçe](README.tr.md)

[![CI](https://github.com/omercagatay/worldcup-2026-simulator/actions/workflows/ci.yml/badge.svg)](https://github.com/omercagatay/worldcup-2026-simulator/actions/workflows/ci.yml)

48 takımlı 2026 FIFA Dünya Kupası için geliştirilmiş tam kapsamlı bir Monte Carlo tahmin uygulaması. Rust tabanlı simülasyon motorunu, React panelini, canlı turnuva verilerini ve isteğe bağlı Kimi senaryo analizini bir araya getirir.

> Bu projenin ürettiği olasılıklar ve adil oranlar model tahminidir; bahis ya da finansal tavsiye değildir.

## Öne çıkanlar

- Rayon ile 100–200.000 turnuva simülasyonunu paralel çalıştırır; panelin varsayılanı 50.000'dir.
- Beklenen gol tahminlerinde Elo, Dixon–Coles ve pi-rating modellerini harmanlar.
- Düşük skorlu sonuçları daha iyi temsil etmek için Dixon–Coles ortak skor dağılımını kullanır.
- Kesinleşmiş grup ve eleme sonuçlarını sabit tutar; yalnızca turnuvanın kalan yollarını simüle eder.
- Açılışta ve ayarlanabilir aralıklarla herkese açık kaynaklardan reyting ve turnuva sonuçlarını yeniler.
- Şampiyonluk, final, yarı final, çeyrek final, son 16, son 32 ve üçüncülük olasılıklarını hesaplar.
- Grup sonuçlarını, temsili turnuva ağacını, olası final eşleşmelerini, yaklaşan maç tahminlerini ve adil ondalık oranları gösterir.
- Doğal dilde yazılan senaryoları Kimi ile doğrulanmış Elo değişikliklerine dönüştürür ve turnuvayı yeniden simüle eder.
- IP bazlı hız sınırları, istek doğrulama, tekrarlanabilir seed değerleri, açık/koyu tema, Docker desteği ve GitHub Actions CI içerir.

## Teknolojiler

| Katman | Teknoloji |
|---|---|
| Backend/API | Rust 1.75+, Axum, Tokio |
| Simülasyon | Rayon, Rand, Dixon–Coles, pi-ratings, Elo/Poisson |
| Frontend | React 18, TypeScript, Vite |
| Canlı veri | World Football Elo Ratings ve İngilizce Wikipedia API'si |
| Senaryo analizi | Moonshot API üzerinden Kimi |
| Dağıtım | Çok aşamalı Docker imajı; Railway uyumlu |

## Model nasıl çalışır?

Saf Elo bileşeni, reyting farkını ve ev sahibi avantajını beklenen gole dönüştürür:

```text
lambda_A = 1.35 × 10^((Elo_A - Elo_B + ev_sahibi_avantajı) / 1600)
lambda_B = 1.35 × 10^(-(Elo_A - Elo_B + ev_sahibi_avantajı) / 1600)
```

Bu değerler varsayılan olarak geçmiş millî maç sonuçlarıyla eğitilen iki modelle harmanlanır:

- **Elo (0,5):** Güncel takım gücü ve ev sahibi için 80 puanlık avantaj.
- **Dixon–Coles (0,3):** Zaman ağırlıklı hücum/savunma güçleri ve düşük skor korelasyonu.
- **Pi-ratings (0,2):** Maç geçmişinden sıralı olarak güncellenen iç/dış saha gücü.

Karışımı değiştirmek için `ENSEMBLE_WEIGHTS` değerini ayarlayın; `1,0,0` saf Elo modelini seçer. Dixon–Coles ağırlığı etkinken normal süre skorları bu modelin ortak dağılımıyla üretilir. Berabere biten eleme maçı bağımsız örneklenen uzatmaya, gerekirse Elo farkının etkisi azaltılmış penaltı atışlarına gider.

Canlı reytingler, elle girilen değişiklikler ve Kimi senaryoları Elo bileşenini günceller. Gömülü Dixon–Coles ve pi-rating parametreleri, geçmiş modeller açıkça yeniden eğitilene kadar değişmez.

Turnuva motoru gruplarda puan, averaj, atılan gol ve ikili averaj kurallarını uygular. Üçüncü sıradaki takımları sıralar ve FIFA'nın uygun son 32 yuvalarına kısıt eşleştirmesiyle yerleştirir. Kesinleşmiş sonuçlar korunduğu için elenen takımlar simüle edilmiş bir yola geri dönemez.

## Yerelde çalıştırma

### Gereksinimler

- Rust 1.75 veya üzeri
- Node.js 20 veya üzeri ve npm

Temel simülasyon için API anahtarı gerekmez. `KIMI_API_KEY` yalnızca doğal dil senaryolarında gereklidir.

### Geliştirme modu

Vite geliştirme sunucusu `/api` isteklerini `3001` portuna yönlendirdiği için backend'i bu portta çalıştırın.

Terminal 1:

```bash
git clone https://github.com/omercagatay/worldcup-2026-simulator.git
cd worldcup-2026-simulator
cp .env.example .env
PORT=3001 cargo run --release
```

Terminal 2:

```bash
cd worldcup-2026-simulator/frontend
npm ci
npm run dev
```

Tarayıcıda <http://localhost:5173> adresini açın. İlk tahmin otomatik başlar.

### Üretime benzer yerel derleme

Önce frontend'i derleyin; Axum daha sonra `frontend/dist` klasörünü API ile birlikte `3000` portundan sunar.

```bash
cd frontend
npm ci
npm run build
cd ..
cargo run --release
```

Tarayıcıda <http://localhost:3000> adresini açın.

## Yapılandırma

`.env.example` dosyasını `.env` adıyla kopyalayın ve gereken değerleri düzenleyin:

| Değişken | Varsayılan | Amaç |
|---|---:|---|
| `KIMI_API_KEY` | ayarlanmamış | `/api/scenario` özelliğini açar; anahtar Moonshot platformundan alınır. |
| `PORT` | `3000` | Backend HTTP portu. Vite geliştirme sunucusuyla `3001` kullanın. |
| `RUST_LOG` | `wc2026_sim=info` | Rust günlük filtresi. |
| `LIVE_REFRESH_MINUTES` | `30` | Canlı veri yenileme aralığı; `0` arka plan yenilemesini kapatır. |
| `ENSEMBLE_WEIGHTS` | `0.5,0.3,0.2` | Virgülle ayrılmış Elo, Dixon–Coles ve pi-rating ağırlıkları. |
| `TRUST_PROXY` | `0` | Hız sınırında `X-Forwarded-For` başlığına yalnızca temizleyen bir ters vekil arkasında güvenin. |

## Panel kullanımı

1. Simülasyon sayısını ve seed değerini seçip **Run** düğmesine basın. Aynı seed, aynı yapılandırmanın tekrarlanabilmesini sağlar.
2. Şampiyonluk tahminlerini, adil oranları, olası finalleri, temsili turnuva ağacını, grup sonuçlarını ve canlı turnuva verilerini inceleyin.
3. Reytingleri ve kesinleşmiş sonuçları hemen yenilemek için **Update live data** düğmesini kullanın.
4. `Fransa'nın as kalecisi finalde oynayamayacak` gibi bir senaryo girin. Kimi etkisini açıklar, doğrulanmış takım reytinglerini üretir ve yeni simülasyonu başlatır.

## API

| Uç nokta | Metot | IP başına sınır | Açıklama |
|---|---|---:|---|
| `/api/health` | `GET` | — | Servis sürümü, model yapılandırması ve son canlı yenileme. |
| `/api/simulate` | `POST` | 30/dk | İsteğe bağlı Elo değerleriyle temel simülasyonu çalıştırır. |
| `/api/scenario` | `POST` | 10/dk | İstemi Kimi ile analiz eder ve üretilen Elo değerleriyle yeniden çalıştırır. |
| `/api/refresh` | `POST` | 5/dk | Güncel reytingleri ve turnuva sonuçlarını indirip uygular. |
| `/api/live` | `GET` | — | Önbellekteki en son canlı veri görüntüsünü döndürür. |
| `/api/upcoming` | `GET` | 30/dk | Eşleşmesi belli, oynanmamış eleme maçlarını tahmin eder. |

Simülasyon istekleri 100–200.000 deneme kabul eder. Senaryo istemleri 2.000 karakterle sınırlıdır. Elo değişiklikleri bilinen bir takım adı kullanmalı ve 1.000–2.600 aralığında olmalıdır. İstek gövdesi sınırı 1 MiB'dir.

### Temel simülasyon

```bash
curl -X POST http://localhost:3000/api/simulate \
  -H 'Content-Type: application/json' \
  -d '{"n_sims":50000,"seed":12345}'
```

### Elle reyting değiştirerek simülasyon

`elo_overrides` puan farklarını değil, yeni reyting değerlerini içerir.

```bash
curl -X POST http://localhost:3000/api/simulate \
  -H 'Content-Type: application/json' \
  -d '{"n_sims":50000,"seed":12345,"elo_overrides":{"Turkey":1825}}'
```

### Doğal dil senaryosu

```bash
curl -X POST http://localhost:3000/api/scenario \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"Fransa’nın as kalecisi finalde oynayamayacak","n_sims":50000,"seed":12345}'
```

## Docker

```bash
docker build -t wc2026-sim .
docker run --rm -p 3000:3000 \
  -e KIMI_API_KEY=anahtarınız \
  wc2026-sim
```

Senaryo analizi gerekmiyorsa `KIMI_API_KEY` satırını kaldırın.

## Railway'e dağıtım

1. Bu GitHub deposundan bir Railway servisi oluşturun.
2. Railway kökteki `Dockerfile` dosyasını algılayıp Rust backend'i ve React frontend'i derler.
3. Senaryo analizi açılacaksa `KIMI_API_KEY` ekleyin.
4. Hız sınırının Railway'in temizleyen uç vekilinden gelen istemci adresini kullanması için `TRUST_PROXY=1` ayarlayın.
5. İsterseniz `LIVE_REFRESH_MINUTES`, `ENSEMBLE_WEIGHTS` ve `RUST_LOG` değerlerini özelleştirin.
6. Sağlık kontrolü yolunu `/api/health` olarak ayarlayın.

Uygulama Railway'in sağladığı `PORT` değerini otomatik okur.

## Doğrulama

GitHub Actions iş akışı aynı temel kontrolleri çalıştırır:

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release

cd frontend
npm ci
npm run build
```

## Geçmiş model verilerini yenileme

Depoda derleme sırasında kullanılan geçmiş sonuçlar ve eğitilmiş Dixon–Coles parametreleri hazır bulunur. Bunları yenileyip modeli yeniden eğitmek için:

```bash
./scripts/refresh_history.sh
cargo run --release --example fit_dc
```

Yeni bir eğitimi commit etmeden önce `data/` altındaki değişiklikleri inceleyip doğrulayın.

## Proje yapısı

```text
.
├── src/
│   ├── main.rs           # Axum sunucusu, yapılandırma ve arka plan yenileme
│   ├── sim.rs            # Turnuva ve paralel Monte Carlo motoru
│   ├── dixoncoles.rs     # Dixon–Coles eğitimi ve ortak skor olasılıkları
│   ├── piratings.rs      # Geçmiş veriye dayalı pi-rating modeli
│   ├── history.rs        # Geçmiş sonuçları yükleme ve takım adı normalleştirme
│   ├── scraper.rs        # Canlı reyting ve turnuva verisi alma
│   ├── handlers.rs       # API işleyicileri
│   ├── llm.rs            # Kimi senaryo analizi
│   ├── models.rs         # API istek ve yanıt türleri
│   ├── validation.rs     # İstek doğrulama
│   └── rate_limit.rs     # IP bazlı hız sınırı
├── data/                 # Geçmiş sonuçlar ve eğitilmiş model parametreleri
├── frontend/             # React ve TypeScript paneli
├── examples/             # Model eğitimi ve duman testi araçları
├── scripts/              # Veri yenileme yardımcıları
├── .github/workflows/    # CI yapılandırması
└── Dockerfile            # Çok aşamalı üretim imajı
```

## Veri ve model sınırlamaları

- Canlı yenileme üçüncü taraf uç noktalarına ve bunların güncel veri/sayfa biçimlerine bağlıdır; yenileme başarısız olursa gömülü temel veri kullanılmaya devam eder.
- Adil oranlar yalnızca simüle edilen olasılıkların tersidir; bahis şirketi marjı, likidite veya piyasa bilgisi içermez.
- Senaryo reytingleri model tarafından üretilen varsayımlardır. Dönen açıklamayı okuyun ve sonucu keşif amaçlı değerlendirin.
- Tahmin kalitesi reytinglere, geçmiş veri kapsamına, model varsayımlarına ve deneme sayısına bağlıdır.
