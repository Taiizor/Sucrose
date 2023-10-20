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
        /// Title boş veya çok uzun!
        /// </summary>
        Title,
        /// <summary>
        /// Author boş değil ve çok uzun!
        /// </summary>
        Author,
        /// <summary>
        /// Source dosyası bulunamadı!
        /// </summary>
        Source,
        /// <summary>
        /// Contact boş değil ve çok uzun!
        /// </summary>
        Contact,
        /// <summary>
        /// Preview dosyası bulunamadı!
        /// </summary>
        Preview,
        /// <summary>
        /// Seçilen ZIP dosyası şifreli!
        /// </summary>
        Encrypt,
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
        /// Contact boş değil, url ve mail değil!
        /// </summary>
        Contact2,
        /// <summary>
        /// Arguments boş değil ve çok uzun!
        /// </summary>
        Arguments,
        /// <summary>
        /// Seçilen dosya .zip uzantılı değil!
        /// </summary>
        Extension,
        /// <summary>
        /// Thumbnail dosyası bulunamadı!
        /// </summary>
        Thumbnail,
        /// <summary>
        /// Geçersiz sistem işlemci değeri!
        /// </summary>
        SystemCpu,
        EmptyInfo,
        /// <summary>
        /// Tema bu uygulamanın daha yüksek bir sürümü için oluşturulmuş!
        /// </summary>
        AppVersion,
        InvalidUrl,
        SystemBios,
        SystemDate,
        SystemAudio,
        Description,
        InvalidInfo,
        TriggerTime,
        StretchMode,
        VolumeLevel,
        ShuffleMode,
        InvalidFile,
        SystemMemory,
        SystemBattery,
        SystemGraphic,
        SystemNetwork,
        EmptyCompatible,
        InvalidExtension,
        PropertyListener,
        SystemMotherboard,
        InvalidCompatible,
        UnforeseenConsequences
    }
}