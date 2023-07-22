namespace Sucrose.Shared.Dependency.Enum
{
    /// <summary>
    /// 
    /// </summary>
    public enum CompatibilityType
    {
        /// <summary>
        /// Tema başarıyla kontrolden geçti.
        /// </summary>
        Pass,
        /// <summary>
        /// Tema bu uygulamanın desteklemediği bir tür için oluşturulmuş!
        /// </summary>
        Type,
        /// <summary>
        /// Source dosyası bulunamadı!
        /// </summary>
        Source,
        /// <summary>
        /// Preview dosyası bulunamadı!
        /// </summary>
        Preview,
        /// <summary>
        /// Seçilen dosya gerçekten ZIP dosyası değil!
        /// </summary>
        ZipType,
        /// <summary>
        /// SucroseInfo.json dosyası bulunamadı!
        /// </summary>
        InfoFile,
        /// <summary>
        /// Seçilen dosya bulunamadı!
        /// </summary>
        NotFound,
        /// <summary>
        /// Geçersiz döngü modu değeri!
        /// </summary>
        LoopMode,
        /// <summary>
        /// Seçilen dosya .zip uzantılı değil!
        /// </summary>
        Extension,
        /// <summary>
        /// Thumbnail dosyası bulunamadı!
        /// </summary>
        Thumbnail,
        /// <summary>
        /// Tema bu uygulamanın daha yüksek bir sürümü için oluşturulmuş!
        /// </summary>
        AppVersion,
        /// <summary>
        /// Geçersiz url adresi!
        /// </summary>
        InvalidUrl,
        /// <summary>
        /// Geçersiz tetikleme zamanı!
        /// </summary>
        TriggerTime,
        /// <summary>
        /// Geçersiz uzatma modu değeri!
        /// </summary>
        StretchMode,
        /// <summary>
        /// Geçersiz ses seviyesi değeri!
        /// </summary>
        VolumeLevel,
        /// <summary>
        /// Geçersiz karıştırma modu değeri!
        /// </summary>
        ShuffleMode,
        /// <summary>
        /// Geçersiz dosya!
        /// </summary>
        InvalidFile,
        /// <summary>
        /// Geçersiz bilgisayar zamanı değeri!
        /// </summary>
        ComputerDate,
        /// <summary>
        /// Geçersiz dosya uzantısı!
        /// </summary>
        InvalidExtension,
        /// <summary>
        /// Geçersiz özellik dinleyici değeri!
        /// </summary>
        PropertyListener,
        /// <summary>
        /// Öngörülemeyen sonuçlar.
        /// </summary>
        UnforeseenConsequences
    }
}