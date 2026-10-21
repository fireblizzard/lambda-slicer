# Türkçe. Gözden geçirilmemiş taslak

app-title = Lambda Slicer

## Araç çubuğu

toolbar-open = Demo aç…
toolbar-add-folder = Klasör ekle…
toolbar-rescan = Yeniden tara
toolbar-cancel = İptal
toolbar-settings = Ayarlar
toolbar-rescan-busy = Zaten bir tarama sürüyor.
toolbar-rescan-nothing = Taramak için bir klasör ekle veya bir demo aç.
toolbar-cancel-idle = Süren bir tarama yok.
dialog-demos = Half-Life demoları

## Ayarlar

settings-title = Ayarlar
settings-language = Dil
settings-unreviewed = Bu çeviri, henüz anadili Türkçe olan biri tarafından gözden geçirilmemiş bir taslaktır.
settings-workers = Aynı anda analiz edilen demolar
settings-workers-hint = Aynı anda daha çok demo analiz etmek daha çabuk biter ama bilgisayarı daha çok yorar.
settings-cache = Kütüphanenin saklandığı yer
settings-folders = Kütüphane klasörleri
settings-folders-empty = Henüz klasör eklenmedi.
settings-remove = Kaldır

## Kütüphane

library-empty = Başlamak için bir klasör ekleyin ya da bir demo açın.
library-no-matches = Bu filtrelere uyan demo yok.
library-no-cache = Bu bilgisayarda kütüphaneyi saklayacak bir yer yok.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demo
}
library-percent = %{ $percent }
library-demos-of = { $total } demodan { $shown }
library-total = Kütüphanede { $count } demo
filters-title = Filtreler
filters-find = Bul
filters-pov = Bakış açısı
filters-quality = Kalite
filters-recorded = Kayıt tarihi
pov-any = Tümü
pov-playing = Oynayan
pov-spectating = İzleyici
help-pov = Oynayan, kaydın oynatmanın en fazla yarısında izleyici olduğu, İzleyici ise yarısından fazlasında izleyici olduğu demolardır.
columns-button = Sütunlar
list-back = Kütüphane tablosuna dön
sort-by = Sırala
sort-ascending = Artan
sort-descending = Azalan
score-more-sides = { $count } taraf daha
intensity-help = Yoğunluk { $score }. Çubuk bunu kütüphanedeki en yüksek değerle, { $max }, karşılaştırır.
value-unknown = Bilinmiyor
path-copy = Dosya konumunu kopyala
path-show = Klasörde göster
path-show-failed = Klasör açılamadı ({ $detail }). Bunun yerine dosya konumunu kopyala.

filter-search = Dosya adlarında ve klasörlerde ara
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
column-action = Yoğunluk
column-spectating = İzleyici

status-complete = Analiz edildi
status-partial = Kısmi
status-failed = Başarısız
status-unsupported = Desteklenmiyor
status-no-use = Silah kullanımı yok

kind-multiplayer-view = Oyuncu görünümü
kind-singleplayer = Tek oyunculu
kind-hltv = HLTV
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
details-action = En yüksek yoğunluk { NUMBER($score, minimumFractionDigits: 2) }

## Tarama

scan-progress = Bulunan { $found }, biten { $done } (zaten bilinen { $cached }, analiz edilen { $analysed }), okunan { $megabytes } MB
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

analysis-panicked = Analiz, Lambda Slicer'daki bir arıza yüzünden durdu: { $detail }
cache-cancelled = Analiz iptal edildi, bu yüzden hiçbir şey kaydedilmedi.
cache-database = Kaydedilen kütüphane kullanılamıyor: { $detail }
cache-directory = Kaydedilen kütüphanenin klasörü oluşturulamıyor: { $detail }
container-bad-magic = Bu bir Half-Life demosu değil.
container-frame-bad-time = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk; karenin zamanı hatalı.
container-frame-missing-terminal = Kaydın { $entry }. bölümü düzgün bitmiyor.
container-frame-negative-length = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk.
container-frame-over-limit = { $entry }. bölümün { $frame }. karesi izin verilenden büyük.
container-frame-truncated = Kayıt, { $entry }. bölümün { $frame }. karesinde kesiliyor.
container-frame-unknown-tag = Kayıt, { $entry }. bölümün { $frame }. karesinde bozuk; karede Lambda Slicer'ın tanımadığı bir şey var.
container-io = Dosya okunamıyor: { $detail }
container-no-directory = Kayıt hiç bitirilmemiş, bu yüzden içindekiler tablosu yok.
container-range = Kaydın içindekiler tablosu dosyanın dışını gösteriyor.
container-too-short = Dosya, demo olmak için fazla kısa.
coverage-hltv = HLTV kaydı. Bunu bir oyuncu kaydetmedi, bu yüzden olaylar HLTV'nin izlediği yeri takip ediyor.
coverage-mostly-spectating = Kaydeden kişi oynatmanın %{ $percent } kadarında izleyiciydi, bu yüzden olayların çoğu, onları izlerken görüldüğü şekliyle diğer oyunculara ait.
coverage-no-local-events = Kaydeden kişinin kendi eylemleri bu demoya kaydedilmedi, bu yüzden yalnızca sunucunun gönderdikleri görünüyor.
coverage-singleplayer = Tek oyunculu. Geri kalan her şey canavarlar ya da haritanın parçaları ve kill feed yok.
coverage-unknown = Kayıt, türünü anlamaya yetecek kadar okunamadı.
coverage-visible-only = Oyun, diğer oyuncuların eylemlerini yalnızca sunucu kaydeden kişinin onları görebildiğini düşündüğü sürece alır, bu yüzden sayıları gerçekte olandan az olabilir.
decode-failed = Kaydın okunması { $time } anında durdu (kare { $frame }): { $detail }
export-active-effect = Başlangıçta { $effect } hâlâ sürüyor ve bir klip bunu henüz aktaramıyor.
export-client-dropped-packets = Oyun { $time } anında çok geç gelen verileri attı; bir klip bunu henüz kopyalayamıyor.
export-cross-segment = { $time } anında, klibin içinde harita değişti ya da oyuncu yeniden bağlandı; bir klip bunun üzerinden geçemez.
export-destination-exists = { $path } zaten var. Başka bir ad seçin.
export-empty-range = Klipte dışa aktarılacak bir şey yok.
export-io = { $path } yazılamıyor: { $detail }
export-missing-delta-base = Kayıtta, klibin ihtiyaç duyduğu başlangıç öncesi veriler eksik (numara { $sequence }).
export-missing-parent = { $path } klasörü yok.
export-range-beyond-end = { $end } bitişi, kaydın sonundan ({ $segment_end }) sonra.
export-same-as-source = Dışa aktarılan dosya, alındığı demonun yerini alırdı.
export-source-changed = { $path } dışa aktarma sırasında değişti.
export-source-decode-failed = Demo okunamıyor: { $detail }
export-source-failed-crosscheck = Kontrol edildiğinde, Lambda Slicer'ın bu demodan çıkardığı sonuç demonun kendisiyle uyuşmadı, bu yüzden ondan alınan kliplere güvenilemez.
export-unknown-persistent-message = Başlangıçtan önceki { $name }, bir klibin henüz aktaramadığı bir şey ayarlıyor.
export-unproven-state = { $time } anında kayıt, tahmin edilmesi gereken verilere dayanıyor, bu yüzden klip tam olarak yapılamıyor.
export-unrepresentable-time-field = Başlangıçtaki bir zaman tam olarak kaydedilemiyor: { $detail }
export-unsupported = Desteklenmiyor: { $detail }
export-validation-failed = Dışa aktarılan klip son kontrolü geçemedi: { $detail }
io-not-found = Dosya ya da klasör bulunamadı.
io-other = { $detail }
io-permission-denied = Erişim reddedildi.
io-unexpected-end = Dosya beklenmedik şekilde bitti.
protocol-unsupported = Bu demo desteklenmeyen bir oyun sürümünden (protokol { $demo }/{ $network }). Yalnızca 5/48 destekleniyor.
protocol-unsupported-network = Sunucu desteklenmeyen bir oyun sürümüne geçti (protokol { $network }).

## Demo görünümü

demo-close = Kapat
demo-loading = { $name } analiz ediliyor…
show-shots = Atışlar
show-actions = Diğer eylemler
show-kills = Öldürmeler
show-switches = Silah değişimleri
lane-unknown = Bilinmeyen oyuncu
lane-recorder = { $name } (kaydeden)
event-kill = { $victim } öldürüldü
event-switch = Silah değişimi
action-swing = Vuruş
action-fire = Atış
action-launch = Fırlatma
action-charge-activity = Şarj
action-charge-stop = Şarj sonu
action-beam-start = Işın başlangıcı
action-beam-sustain = Işın
action-beam-stop = Işın sonu
action-placement-attempt = Yerleştirme denemesi
action-release-attempt = Bırakma denemesi
mode-primary = birincil
mode-charged = şarjlı
mode-zoomed = dürbünlü
mode-unzoomed = dürbünsüz
mode-grenade = el bombası
mode-bullets = mermi
mode-single = tekli
mode-double = çift
mode-rapid = seri
demo-events = { $count ->
    [one] { $count } olay
   *[other] { $count } olay
}

column-date = Tarih
column-score = Skor
filter-from = Başlangıç
filter-to = Bitiş
date-from-name = Dosya adından alınan tarih
date-from-modified = Dosyanın son değiştirilme tarihi; kopyalama ya da indirme bunu değiştirebilir
timeline-best-action = Altın dakika
timeline-window = { $rank }. { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Son skorlar, son öldürmede { $time }
scores-at = { $time } anındaki skorlar
scores-show-final = Son skorları göster
scores-frags = Fraglar
scores-deaths = Ölümler

viewer-play = Oynat
viewer-pause = Duraklat
viewer-volume = Ses düzeyi
viewer-no-sound = Ses yok
viewer-play-backward = Geri sar
viewer-step-back = Önceki kare
viewer-step-forward = Sonraki kare
viewer-skip-back = 10 saniye geri
viewer-skip-forward = 10 saniye ileri
viewer-rate = { $rate }×
viewer-shortcut = Kısayol: { $key }
key-space = Boşluk
key-shift-left = Shift+Sol
key-shift-right = Shift+Sağ
viewer-map-failed = { $map } haritası yüklenemedi
viewer-stopped = Oynatma erken durdu: { $reason }
viewer-missing = Okunamayan dosyalar: { $count }
event-previous = Önceki olay
event-next = Sonraki olay
event-play = Olayı oynat
timeline-title = Zaman çizelgesi
timeline-lanes = Oyuncu satırları
timeline-zoom-in = Yakınlaştır
timeline-zoom-out = Uzaklaştır
timeline-fit-all = Tüm demoyu sığdır
timeline-gestures = Olayını seçmek için bir işarete, oraya gitmek için başka herhangi bir yere tıklayın. Demoda ilerlemek için üstteki zamanlar boyunca ya da şu anki anı gösteren çizgiyi sürükleyin. Görünümü kaydırmak için başka herhangi bir yeri sürükleyin. Farenin olduğu yerde yakınlaştırıp uzaklaştırmak için tekerleği çevirin, tüm demoyu görmek için çift tıklayın. Klibin nerede başlayıp bittiğini değiştirmek için kenarını sürükleyin, düzenlemek için klibe tıklayın.
timeline-density = Gölgeli çubuklar, gösterilen olaylardan her noktada kaç tane olduğunu demonun en yoğun noktasına göre gösterir.
timeline-density-count = { $count ->
    [one] Burada gösterilen bir olay
   *[other] Burada gösterilen { $count } olay
}
timeline-session-count = Bölüm { $session }: { $count }
timeline-choose = Bu noktadaki olaylar
legend-clip = Klip
event-session = Bölüm { $session }
event-unreachable = Bu olay, demonun saatinin baştan başladığı bir harita değişikliğinden ya da yeniden bağlanmadan sonra geliyor, bu yüzden görüntüleyici ona gidemez.
event-hidden = Seçili olay filtrelerce gizleniyor.
highlight-jump = Öne çıkan ana git…
highlight-help = Öne çıkan anlar, filtreler ne gösterirse göstersin tüm demoyu aksiyonuna göre sıralar.
kills-with = Öldürme filtresi
refine-midair = Midair
refine-speed = Hız
refine-wallgauss = Wallgauss
refine-clear = Temizle
refine-airborne = Havada
refine-either = Herhangi biri
midair-both = İkisi de
midair-one = Yalnızca biri
help-midair-either = Öldüren, kurban ya da ikisi de havadaydı. Öldüren, öldürmeye bağlı atış anında, böyle bir atış yoksa öldürme anında; kurban ise ölüm anında kontrol edilir. Yerden 50 birimden az yükselen sıradan bir zıplama sayılmaz. Zeminin nerede olduğunu bilmek için Half-Life klasörünün seçilmiş olması gerekir.
help-midair-both = Öldüren de kurban da, her biri kontrol edildiği anda, havadaydı.
help-midair-one = Tam olarak biri havadaydı ve diğerinin yerde olduğu biliniyor.
speed-at-least = En az
speed-unit = ups
speed-killer = Öldüren
speed-victim = Kurban
help-speed = Zemin boyunca hız, ups cinsinden; hız göstergesindeki gibi yukarı ve aşağı hareket sayılmaz. Öldüren, öldürmeye bağlı atış anında, böyle bir atış yoksa öldürme anında; kurban ise ölüm anında ölçülür. Diğer oyuncuların hızları tahminidir.
wallgauss-weapon = Wallgauss yalnızca gauss öldürmelerini kapsar ve silah filtresi { $weapon } gösteriyor.
wallgauss-checking = Wallgauss kontrol ediliyor… { $examined } / { $total } öldürme
wallgauss-needs-files = Wallgauss, haritanın duvarlarının nerede olduğunu bilmek için oyun dosyalarına ihtiyaç duyar.
wallgauss-failed = Wallgauss kontrol edilemedi. { $reason }
help-wallgauss = Duvarın içinden geçen gauss öldürmeleri; her atış, sunucunun yaptığı gibi harita boyunca takip edilerek bulunur. Karara bağlanamayan bir öldürme, örneğin kurbanın kaydın göremediği bir yerde olduğu durum, bilinmiyor sayılır ve dışarıda bırakılır.
badge-wall = WG
wall-yes-exit = Wallgauss, atıştan çıkarıldı: ışının { $thickness } birim kalınlığındaki bir duvardan çıktığı yerdeki patlama.
wall-yes-beyond = Wallgauss, atıştan çıkarıldı: ışın, { $thickness } birim kalınlığındaki bir duvardan geçtikten sonra isabet etti.
wall-limits = Sunucu bunu kaydetmediği için atıştan ve oyuncuların bulunduğu yerlerden çıkarıldı. Bir oyuncu 32 birim genişliğindedir ve kutu sayılır; kurban, demonun gösterdiği yerden bir iki vücut genişliği uzakta olabilir.
wall-no-direct = Wallgauss değil: ışın kurbana herhangi bir duvardan önce isabet edebilirdi.
wall-no-reflection = Wallgauss değil: bir sekmenin patlaması kurbana herhangi bir duvardan önce isabet edebilirdi.
wall-doubt-no-shot = Wallgauss bilinmiyor: öldürenin atışı kayıtta yok.
wall-doubt-no-map = Wallgauss bilinmiyor: haritanın dosyaları okunamadı.
wall-doubt-unseen = Wallgauss bilinmiyor: kurban kaydın göremediği bir yerdeydi.
wall-doubt-brush = Wallgauss bilinmiyor: ışın bir kapıya, asansöre ya da haritanın başka özel bir parçasına çarptı; bunlar kontrol edilmez.
wall-doubt-ambiguous = Wallgauss bilinmiyor: atış hem duvarın içinden hem de duvarsız öldürmüş olabilir.
wall-doubt-unexplained = Wallgauss bilinmiyor: atışın kurbana nasıl ulaştığı bulunamadı.
wall-doubt-unconfirmed = Wallgauss bilinmiyor: atış kurbana yalnızca duvardan geçerek ulaşmış olabilir, ama bu kesin değil.
wall-doubt-not-reached = Wallgauss bilinmiyor: kayıt atıştan önce bitiyor.
wall-pending = Wallgauss henüz kontrol edilmedi.
events-unknown = { $count ->
    [one] Bir öldürmede karar vermek için veri yok
   *[other] { $count } öldürmede karar vermek için veri yok
}
help-unknown = Kaydın göremediği hareket asla yerde ya da duruyor sayılmaz, bilinmiyor sayılır; bu yüzden bu öldürmeler dışarıda bırakılır.
clip-it = Klip yap
badge-headshot = KV
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = intihar
badge-teamkill = takım öldürme
badge-world = dünya
air-both = Kontrol anında ikisi de havadaydı.
air-killer = Kontrol anında öldüren havadaydı.
air-victim = Kontrol anında kurban havadaydı.
air-other-unknown = Diğerinin havada olup olmadığı bilinmiyor.
speed-horizontal = Zemine paralel yatay hız.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Öldüren
motion-who-victim = Kurban
motion-air-yes = havada
motion-air-no = yerde
motion-air-unknown = havada olup olmadığı bilinmiyor
motion-speed-unknown = hız bilinmiyor
motion-exact = Oyun tarafından kaydedildi
motion-trajectory = Hareketten tahmin edildi
motion-unclear = Hareket belirsiz
motion-not-visible = Görüş dışında
motion-at-shot = bağlı atış anında
motion-at-kill = ölüm anında
motion-none = Analizde bu öldürme için hareket yok.
legend-shot = Atış
legend-charge = Şarj
legend-kill = Öldürme
legend-death = Ölüm
legend-switch = Silah değişimi

column-headshots = Kafa vuruşları
filter-headshots = Kafadan öldürmeler
filter-midair = Midair öldürmeler
event-headshot = kafadan
scores-headshots = KV

column-accuracy = İsabet
filter-min-accuracy = En az isabet (%)
scores-accuracy = İsabet

help-headshots = Kafaya isabetle yapılan öldürmeler; kurbanın ölüm animasyonundan anlaşılır. Patlamalar sayılmaz. Yalnızca kaydedenin öldürmeleri gösterilir, çünkü diğer oyuncular çoğu zaman kaydın göremediği yerlerde öldürür. Düelloda rakibinkiler de gösterilir, çünkü rakibin tüm öldürmeleri kaydedene karşıdır.
help-headshots-column = Kaydedenin kafaya isabetle yaptığı öldürmeler; kurbanın ölüm animasyonundan anlaşılır. Patlamalar sayılmaz. Diğer oyuncuların öldürmeleri sayılmaz, çünkü çoğu zaman kaydın göremediği yerlerde olur.
help-accuracy = İsabet eden atışların oranı; bir isabetin nişan hattında bıraktığı kana göre, rastlantısal eşleşmeler çıkarılarak hesaplanır. Glock, python, MP5 mermileri, pompalı, dürbünlü arbalet ve gauss ışınının doğrudan isabetleri sayılır. Patlamalar ve gauss sekmeleri kan bırakmaz, sayılmaz. Yalnızca kaydeden için, en az 50 atıştan sonra gösterilir.
help-headshot-filter = Herhangi bir oyuncunun, kayıtta görülen en az bir kafadan öldürmesi olan demolar.
help-midair-filter = Öldürenin ya da kurbanın havada, yerden en az 50 birim yukarıda olduğu en az bir öldürme bulunan demolar. Half-Life klasörü seçilmemişse sıradan bir zıplama da sayılır.
help-accuracy-filter = Kaydedenin atışlarının en az bu kadarını isabet ettirdiği demolar.

clips-new = Yeni klip
clips-select-event = Önce listeden ya da zaman çizelgesinden bir olay seçin.
clips-other-session = Burada klip başlayamaz, çünkü bir harita değişikliğinden ya da yeniden bağlanmadan sonra demonun saati baştan başladı ve daha önceki bir bölüm aynı zamanlara sahip.
clips-add-window = Bu yoğun anın klibi
clip-default-name = Klip { $number }
clip-name = Ad
clip-length = Süre { $duration }
clip-remove = Kaldır
clip-loop = Döngü
clip-time-help = Saniye (96), dakika ve saniye (1:36) ya da saat, dakika ve saniye (00:01:36.250)
clips-empty-hint = Henüz klip yok. Bir olay seçip Klip yap’a basın ya da şu anki andan bir klip başlatmak için Yeni klip’e basın.
clips-new-help = Şu anki andan başlayan on saniyelik bir klip.
clip-in = Başlangıç
clip-out = Bitiş
clip-set-in = Klibi şu anki andan başlat
clip-set-out = Klibi şu anki anda bitir
clip-no-playhead = Görüntüleyici kullanılamıyor, bu yüzden şu anki an alınamıyor.
clip-play = Klibi bir kez oynat
clip-show-on-timeline = Zaman çizelgesinde göster
export-destination = Dışa aktarma klasörünü değiştir
export-skipped = Atlandı. { $reason }
export-earlier-range = Bu dışa aktarma { $start } – { $end } aralığına, yani klibin siz değiştirmeden önceki hâline ait.
scores-player = Oyuncu
scores-none = Kayıtta skor yok.
clip-effective = Dışa aktarma, kaydın izin verdiği kadar yakın şekilde { $start } – { $end } ({ $duration }) arasını kapsar.
settings-clip-padding = Klip payı
settings-preroll = Olaydan önceki saniye
settings-postroll = Olaydan sonraki saniye
export-start-first-group = Bir klip kaydın en başından başlayamaz. Biraz daha sonradan başlatın.
export-start-during-loading = Başlangıç, oyun hâlâ bir harita yüklerken. Biraz daha sonradan başlatın.

export-all = Tüm klipleri dışa aktar
export-one = Dışa aktar
export-cancel = Dışa aktarmayı durdur
export-folder = Dışa aktarılanlar { $folder } klasörüne gider.
export-folder-beside = Dışa aktarılanlar demonun yanına, { $folder } klasörüne gider.
export-waiting = Dışa aktarılmayı bekliyor…
export-running = Dışa aktarılıyor… Bu, demonun tamamını okur ve biraz sürebilir.
export-cancelled = Dışa aktarma durdurulduğu için aktarılmadı.
export-saved = Kaydedildi: { $path }. Orijinalle karşılaştırılan an: { $count }.
export-playdemo = Oyun konsolunda:
export-copy-first = { $name } dosyasını liblist.gam bulunan mod klasörüne kopyalayın, sonra oyun konsolunda:
export-copy = Kopyala
settings-export-folder = Dışa aktarma klasörü
settings-export-choose = Seç…
settings-file-names = Dosya adları
settings-clip-name-files = Dosya için klibin adını kullan
settings-demo-name-first = Demonun adıyla başla
settings-viewer = Demo görüntüleyici
settings-viewer-game = Half-Life klasörü
settings-viewer-game-none = Seçilmedi
settings-viewer-game-hint = valve klasörünü içeren klasör. Görüntüleyici demoları buradaki oyun haritaları, modelleri ve sesleriyle oynatır.
settings-executable = Oyun programı
settings-executable-hint = Half-Life klasörünü bulmaya yarar. Görüntüleyici oyunun dosyalarını kendisi okur ve oyunu asla başlatmaz.
settings-executable-not-set = Ayarlanmadı, bu yüzden görüntüleyicide ne görüntü ne ses var
settings-executable-optional = Ayarlanmadı ve Half-Life klasörü oyunu içerdiği sürece gerekmiyor
settings-executable-missing = { $path } artık yok. Görüntüleyici Half-Life klasörünü okumaya devam ediyor.
settings-game-override = Lambda Slicer --game ile başlatıldığı için bu sefer şunu kullanıyor: { $path }. Buradaki seçim bir sonraki açılışta geçerli olur.
settings-browse = Gözat…
dialog-executables = Programlar
setup-title = Demo görüntüleyiciyi kur
setup-body = Görüntü ve ses için Half-Life oyun dosyaları gerekir. Oyunun kurulu olduğu yeri Lambda Slicer'a göster, dosyaları oradan okusun.
setup-locate-exe = hl.exe'yi bul…
setup-locate-folder = Half-Life'ı bul…
setup-settings = Ayarlar
setup-still-works = Zaman çizelgesi, olaylar ve klipler onlar olmadan da çalışır.
setup-relocate = { $folder } artık Half-Life içermiyor. Yeniden bul.
setup-not-executable = { $name } { $executable } değil. Kısayolu değil, oyunun kendi programını seç.
setup-not-installed = { $folder } valve klasörü olan bir Half-Life kurulumu içermiyor.
tab-events = Olaylar
tab-clips = Klipler
tab-scoreboard = Skor tablosu
list-hide = Demo listesini gizle
list-show = Demo listesini göster
demo-details = Ayrıntılar
viewer-start = Başa dön
viewer-mute = Sesi kapat
viewer-unmute = Sesi aç
viewer-picture = Görüntü ve görüntüleyici ayarları
viewer-needs-files = Oyunun dosyaları gerekir
viewer-no-sound-why = Ses yok: { $reason }
settings-tab-general = Genel
settings-tab-library = Kütüphane
settings-tab-clips = Klipler
settings-section-game = Half-Life
settings-section-view = Görünüm
settings-language-system = Bilgisayarın dili ({ $name })
settings-section-folders = Klasörler
settings-folders-hint = Kütüphanenin demo aradığı klasörler, alt klasörler dahil.
settings-clip-padding-hint = Bir olay etrafında oluşturulan klibin olaydan önce ve sonra ne kadar tuttuğu.
settings-section-picture = Görüntü
settings-section-sound = Ses
settings-gamma = Gama
settings-brightness = Parlaklık
settings-texgamma = Doku gaması
settings-lightgamma = Işık gaması
settings-overbright = Aşırı parlak aydınlatma
settings-gamma-hint = Oyunun aynı adlı ayarları (gamma, brightness, texgamma, lightgamma, gl_overbright). Aynı parlaklığı görmek için oyundaki gibi ayarlayın.
settings-gamma-defaults = Oyunun varsayılanları
settings-fov-recorded = Kayıttaki görüş alanı
settings-fov = Görüş alanı
settings-viewmodel = Silahı elde göster
settings-see-through = Oyuncuları duvarların ardından gör
settings-see-through-items = Mermiler ve eşyalar gibi diğer modeller de
settings-see-through-hint = Duvarların arkasındaki oyuncular, takım oyunlarının takıma göre belirlediği üst renklerinde siluet olarak görünür. Yalnızca demonun içerdiği görünebilir. Sunucu kaydedenin yakınındaki oyuncuları gönderir; kaydedenin göremediği oyuncuları gizleyen sunucular gösterecek bir şey bırakmaz.
viewer-see-through = Duvar arkasını gör
settings-export-beside = Her demonun yanında
settings-export-hint = liblist.gam bulunan mod klasörünüzde, örneğin Half-Life\ag, klipler tek bir playdemo komutuyla oynatılır.
settings-file-names-hint = Klibin adını kullanmadığınız sürece, dışa aktarılan bir klip demosuna ve başlangıç ile bitiş zamanlarına göre adlandırılır. Oyun yalnızca adı İngilizce harf, rakam, - ve . karakterlerinden oluşan dosyaları açabilir, bu yüzden diğer her şey _ olur. Klip adında bunlardan hiçbiri yoksa dosya bunun yerine zamanlarına göre adlandırılır.

clips-store-moved = { $path } klip dosyası okunamadı, bu yüzden { $moved } olarak saklandı ve klipler boş başlıyor. { $detail }
clips-store-unreadable = { $path } klip dosyası okunamadı: { $detail }. Lambda Slicer kapatılana kadar klipler kaydedilmiyor, böylece dosyadaki hiçbir şey kaybolmuyor.
clips-save-failed = Klipler { $path } konumuna kaydedilemedi: { $detail }
notice-dismiss = Kapat
