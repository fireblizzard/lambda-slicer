# Türkçe. Gözden geçirilmemiş taslak

app-title = Lambda Slicer

## Araç çubuğu

toolbar-open = Demo aç…
toolbar-add-folder = Klasör ekle…
toolbar-rescan = Yeniden tara
toolbar-cancel = İptal
toolbar-settings = Ayarlar
dialog-demos = Half-Life demoları

## Ayarlar

settings-title = Ayarlar
settings-language = Dil
settings-unreviewed = Bu çeviri, henüz anadili Türkçe olan biri tarafından gözden geçirilmemiş bir taslaktır.
settings-workers = Analiz iş parçacıkları
settings-workers-hint = Daha fazla iş parçacığı daha hızlı analiz eder ama işlemciyi daha çok kullanır.
settings-cache = Kütüphane önbelleği
settings-folders = Kütüphane klasörleri
settings-folders-empty = Henüz klasör eklenmedi.
settings-remove = Kaldır

## Kütüphane

library-empty = Başlamak için bir klasör ekleyin ya da bir demo açın.
library-no-matches = Bu filtrelere uyan demo yok.
library-no-cache = Bu sistemde kütüphane önbelleğini saklayacak bir yer yok.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demo
}
library-percent = %{ $percent }

filter-search = Yollarda ara
filter-map = Harita
filter-player = Oyuncu
filter-weapon-any = Herhangi bir silah
filter-min-minutes = En az (dakika)
filter-spectating-any = Oynayan ya da izleyen
filter-spectating-mostly = Çoğunlukla izleyici
filter-spectating-playing = Çoğunlukla oynayan
filter-clear = Filtreleri temizle

column-status = Durum
column-name = Demo
column-map = Harita
column-duration = Süre
column-gauss = Gauss
column-kills = Öldürme
column-action = Aksiyon
column-spectating = İzleyici

status-complete = Analiz edildi
status-partial = Kısmi
status-failed = Başarısız
status-unsupported = Desteklenmiyor
status-no-use = Silah kullanımı yok

kind-multiplayer-view = Oyuncu görünümü
kind-singleplayer = Tek oyunculu
kind-hltv = İzleyici proxy'si
kind-unknown = Bilinmeyen tür

details-recorder = Kaydeden: { $name }
details-shots = { $count ->
    [one] { $count } atış
   *[other] { $count } atış
}
details-kills = { $count ->
    [one] { $count } öldürme
   *[other] { $count } öldürme
}
details-action = En iyi aksiyon puanı { NUMBER($score, minimumFractionDigits: 2) }

## Tarama

scan-progress = Bulunan { $found }, biten { $done } (önbellekten { $cached }, analiz edilen { $analysed }), okunan { $megabytes } MB
scan-cancelling = Durduruluyor…
scan-cancelled = Durduruldu, taranacak { $left } kaldı.
scan-finished = Bitti: { $done } demo, { $analysed } analiz, { $seconds } sn.
scan-problems = { $count ->
    [one] { $count } sorun
   *[other] { $count } sorun
}
problems-title = Tarama sorunları
problem-changed = Dosya analiz edilirken değişti, bu yüzden hiçbir şey kaydedilmedi.
drop-hint = Eklemek için demoları ya da klasörleri bırakın
drop-not-demo = Demo değil: { $names }

## Analiz çekirdeğinden gelen iletiler, koda göre

analysis-panicked = Analiz bir iç hata yüzünden durdu: { $detail }
cache-cancelled = Analiz iptal edildi, bu yüzden hiçbir şey kaydedilmedi.
cache-database = Kütüphane önbelleği kullanılamıyor: { $detail }
cache-directory = Önbellek klasörü oluşturulamıyor: { $detail }
container-bad-magic = Bu bir Half-Life demosu değil.
container-frame-bad-time = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk; karenin zamanı geçersiz.
container-frame-missing-terminal = Kaydın { $entry }. bölümü kapanış kaydı olmadan bitiyor.
container-frame-negative-length = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk.
container-frame-over-limit = { $entry }. bölümün { $frame }. karesi izin verilenden büyük.
container-frame-truncated = Kayıt, { $entry }. bölümün { $frame }. karesinde kesiliyor.
container-frame-unknown-tag = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk; kare türü bilinmiyor.
container-io = Dosya okunamıyor: { $detail }
container-no-directory = Kayıt hiç bitirilmemiş, bu yüzden içindekiler tablosu yok.
container-range = Kaydın içindekiler tablosu dosyanın dışını gösteriyor.
container-too-short = Dosya, demo olmak için fazla kısa.
coverage-hltv = İzleyici proxy kaydı. Kaydeden bir oyuncu yok ve olaylar proxy'nin görünümünü izliyor.
coverage-mostly-spectating = Kaydeden kişi oynatmanın %{ $percent } kadarında izleyiciydi, bu yüzden olayların çoğu, izlenen görünümün gösterdiği şekliyle diğer oyunculara ait.
coverage-no-local-events = Yerel olay yok. Kaydeden kişinin kendi eylemleri, sunucunun gönderdikleri dışında eksik.
coverage-singleplayer = Tek oyunculu. Diğer aktörler canavarlar ya da harita nesneleri ve öldürme akışı yok.
coverage-unknown = Kayıt, türünü anlamaya yetecek kadar çözülemedi.
coverage-visible-only = Diğer oyuncuların olayları bu istemciye yalnızca sunucu onları görünür saydığı sürece ulaşır, bu yüzden sayıları alt sınırdır.
decode-failed = Çözme { $time } anında durdu (kare { $frame }): { $detail }
export-active-effect = Başlangıçta { $effect } sürüyor ve henüz aktarılamıyor.
export-client-dropped-packets = Oyun { $time } anında geç kalan paketleri attı; bir kesim bunu henüz yeniden üretemiyor.
export-cross-segment = Aralığın içinde, { $time } anında yeni bir oturum başlıyor.
export-destination-exists = { $path } zaten var. Başka bir ad seçin.
export-empty-range = Seçilen aralıkta dışa aktarılacak bir şey yok.
export-io = { $path } yazılamıyor: { $detail }
export-missing-delta-base = Kayıtta başlangıçtan önce gereken bir durum eksik (sıra { $sequence }).
export-missing-parent = { $path } için klasör yok.
export-range-beyond-end = { $end } bitişi, kaydın sonundan ({ $segment_end }) sonra.
export-same-as-source = Hedef, kaynak demonun kendisi.
export-source-changed = { $path } dışa aktarma sırasında değişti.
export-source-decode-failed = Kaynak demo çözülemiyor: { $detail }
export-source-failed-crosscheck = Kaynak demo çapraz denetimi geçemedi, bu yüzden yeniden kurulan durumuna güvenilmiyor.
export-unknown-persistent-message = Başlangıçtan önceki { $name }, bir kesimin henüz aktaramadığı bir durum ayarlıyor.
export-unproven-state = { $time } anındaki durum çıkarımla bulunmuş bir tabana dayanıyor ve kanıtlanmadı.
export-unrepresentable-time-field = Başlangıçtaki bir zaman aslına uygun yazılamıyor: { $detail }
export-unsupported = Desteklenmiyor: { $detail }
export-validation-failed = Dışa aktarılan dosya doğrulamayı geçemedi: { $detail }
io-not-found = Dosya ya da klasör bulunamadı.
io-other = { $detail }
io-permission-denied = Erişim reddedildi.
io-unexpected-end = Dosya beklenmedik şekilde bitti.
protocol-unsupported = { $demo }/{ $network } demo protokolü desteklenmiyor, yalnızca 5/48.
protocol-unsupported-network = Sunucu desteklenmeyen { $network } ağ protokolüne geçti.
