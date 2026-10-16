# Türkçe. Gözden geçirilmemiş taslak

app-title = Lambda Slicer

toolbar-open = Demo aç…
toolbar-add-folder = Klasör ekle…
toolbar-rescan = Yeniden tara
toolbar-cancel = İptal
toolbar-settings = Ayarlar

settings-title = Ayarlar
settings-language = Dil
settings-unreviewed = Bu çeviri, henüz anadili Türkçe olan biri tarafından gözden geçirilmemiş bir taslaktır.
settings-workers = Analiz iş parçacıkları
settings-workers-hint = Daha fazla iş parçacığı daha hızlı analiz eder ama işlemciyi daha çok kullanır.
settings-cache = Kütüphane önbelleği
settings-folders = Kütüphane klasörleri
settings-folders-empty = Henüz klasör eklenmedi.
settings-remove = Kaldır
settings-close = Kapat

library-empty = Başlamak için bir klasör ekleyin ya da bir demo açın.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demo
}
library-kills = { $count ->
    [one] { $count } öldürme
   *[other] { $count } öldürme
}
library-action = Aksiyon puanı { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = İzleyici %{ $percent }
