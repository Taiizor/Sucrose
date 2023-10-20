using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHF = Sucrose.Shared.Theme.Helper.Filter;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSZHZ = Sucrose.Shared.Zip.Helper.Zip;

namespace Sucrose.Shared.Zip.Helper
{
    internal static class Archive
    {
        public static SSDECT Check(string Archive)
        {
            try
            {
                // Seçilen dosya var mı?
                if (!File.Exists(Archive))
                {
                    return SSDECT.NotFound;
                }

                // Seçilen dosya .zip uzantılı değil mi?
                if (Path.GetExtension(Archive) != ".zip")
                {
                    return SSDECT.Extension;
                }

                // Seçilen dosya gerçekten ZIP dosyası mı?
                if (!SSZHZ.CheckArchive(Archive))
                {
                    return SSDECT.ZipType;
                }

                //// Seçilen ZIP dosyası şifreli mi?
                //if (SSZHZ.EncryptedArchive(Archive))
                //{
                //    return SSDECT.Encrypt;
                //}

                // Arşivde SucroseInfo.json dosyası var mı?
                if (!SSZHZ.CheckFile(Archive, SMR.SucroseInfo))
                {
                    return SSDECT.InfoFile;
                }

                // Arşivdeki SucroseInfo.json dosyasını okuma
                string Salt = SSZHZ.ReadFile(Archive, SMR.SucroseInfo);

                // Arşivdeki SucroseInfo.json dosyası boş mu?
                if (string.IsNullOrEmpty(Salt))
                {
                    return SSDECT.Empty;
                }

                // Arşivdeki SucroseInfo.json dosyası uygunluk  kontrolü
                bool Json = SSTHI.CheckJson(Salt);

                // Arşivdeki SucroseInfo.json dosyası geçerli mi?
                if (!Json)
                {
                    return SSDECT.Invalid;
                }

                // Arşivdeki SucroseInfo.json dosyasını Info sınıfına dönüştürme
                SSTHI Info = SSTHI.FromJson(Salt);

                // Info içindeki Title değeri boş mu veya 50 karakterden uzun mu?
                if (string.IsNullOrEmpty(Info.Title) || Info.Title.Length > 50)
                {
                    return SSDECT.Title;
                }

                // Info içindeki Description değeri boş mu veya 500 karakterden uzun mu?
                if (string.IsNullOrEmpty(Info.Description) || Info.Description.Length > 500)
                {
                    return SSDECT.Description;
                }

                // Info içindeki Author değeri boş değil ve 50 karakterden uzun mu?
                if (!string.IsNullOrEmpty(Info.Author) && Info.Author.Length > 50)
                {
                    return SSDECT.Author;
                }

                // Info içindeki Contact değeri boş değil ve 250 karakterden uzun mu?
                if (!string.IsNullOrEmpty(Info.Contact) && Info.Contact.Length > 250)
                {
                    return SSDECT.Contact;
                }

                // Info içindeki Contact değeri boş değil, Url ve Mail değil mi?
                if (!string.IsNullOrEmpty(Info.Contact) && !SSTHV.IsUrl(Info.Contact) && !SSTHV.IsMail(Info.Contact))
                {
                    return SSDECT.Contact2;
                }

                // Info içindeki Arguments değeri boş değil ve 250 karakterden uzun mu?
                if (!string.IsNullOrEmpty(Info.Arguments) && Info.Arguments.Length > 250)
                {
                    return SSDECT.Arguments;
                }

                // Info içindeki Thumbnail dosyası var mı?
                if (!SSZHZ.CheckFile(Archive, Info.Thumbnail))
                {
                    return SSDECT.Thumbnail;
                }

                // Info içindeki Preview dosyası var mı?
                if (!SSZHZ.CheckFile(Archive, Info.Preview))
                {
                    return SSDECT.Preview;
                }

                // Info içindeki AppVersion sürümü bu uygulamanın düşük mü?
                if (Info.AppVersion.CompareTo(SHV.Entry()) > 0)
                {
                    return SSDECT.AppVersion;
                }

                // Info içindeki Type değeri bu uygulamanın Type enum değerinden büyük mü?
                if ((int)Info.Type >= Enum.GetValues(typeof(SSDEWT)).Length)
                {
                    return SSDECT.Type;
                }

                // Info içindeki Type değerine göre dosya veya url kontrolü
                if (Info.Type == SSDEWT.Web)
                {
                    if (!SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.WebExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.Url && !SSTHV.IsUrl(Info.Source))
                {
                    return SSDECT.InvalidUrl;
                }
                else if (Info.Type == SSDEWT.Gif)
                {
                    if (!SSTHV.IsUrl(Info.Source) && !SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.GifExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.Video)
                {
                    if (!SSTHV.IsUrl(Info.Source) && !SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.VideoExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SSDEWT.YouTube && !SSTHV.IsYouTube(Info.Source) && !SSTHV.IsYouTubeMusic(Info.Source))
                {
                    return SSDECT.InvalidUrl;
                }
                else if (Info.Type == SSDEWT.Application)
                {
                    if (!SSZHZ.CheckFile(Archive, Info.Source))
                    {
                        return SSDECT.Source;
                    }
                    else if (!SSTHF.AppExtension(Info.Source))
                    {
                        return SSDECT.InvalidExtension;
                    }
                }

                // Arşivde SucroseProperties.json dosyası var mı?
                if (SSZHZ.CheckFile(Archive, SMR.SucroseProperties))
                {
                    // Arşivdeki SucroseProperties.json dosyasını okuma
                    SSTHP Properties = SSTHP.FromJson(SSZHZ.ReadFile(Archive, SMR.SucroseProperties));

                    // Properties içindeki PropertyListener değeri boş değil ve {0} veya {1} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.PropertyListener) && (!Properties.PropertyListener.Contains("{0}") || !Properties.PropertyListener.Contains("{1}")))
                    {
                        return SSDECT.PropertyListener;
                    }
                }

                // Arşivde SucroseCompatible.json dosyası var mı?
                if (SSZHZ.CheckFile(Archive, SMR.SucroseCompatible))
                {
                    // Arşivdeki SucroseCompatible.json dosyasını okuma
                    SSTHC Compatible = SSTHC.FromJson(SSZHZ.ReadFile(Archive, SMR.SucroseCompatible));

                    // Compatible içindeki TriggerTime değeri 1'den küçük mü?
                    if (Compatible.TriggerTime <= 0)
                    {
                        return SSDECT.TriggerTime;
                    }

                    // Compatible içindeki LoopMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.LoopMode) && !Compatible.LoopMode.Contains("{0}"))
                    {
                        return SSDECT.LoopMode;
                    }

                    // Compatible içindeki VolumeLevel değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.VolumeLevel) && !Compatible.VolumeLevel.Contains("{0}"))
                    {
                        return SSDECT.VolumeLevel;
                    }

                    // Compatible içindeki ShuffleMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.ShuffleMode) && !Compatible.ShuffleMode.Contains("{0}"))
                    {
                        return SSDECT.ShuffleMode;
                    }

                    // Compatible içindeki StretchMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.StretchMode) && !Compatible.StretchMode.Contains("{0}"))
                    {
                        return SSDECT.StretchMode;
                    }

                    // Compatible içindeki SystemCpu değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemCpu) && !Compatible.SystemCpu.Contains("{0}"))
                    {
                        return SSDECT.SystemCpu;
                    }

                    // Compatible içindeki SystemBios değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemBios) && !Compatible.SystemBios.Contains("{0}"))
                    {
                        return SSDECT.SystemBios;
                    }

                    // Compatible içindeki SystemAudio değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemAudio) && !Compatible.SystemAudio.Contains("{0}"))
                    {
                        return SSDECT.SystemAudio;
                    }

                    // Compatible içindeki SystemDate değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemDate) && !Compatible.SystemDate.Contains("{0}"))
                    {
                        return SSDECT.SystemDate;
                    }

                    // Compatible içindeki SystemMemory değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemMemory) && !Compatible.SystemMemory.Contains("{0}"))
                    {
                        return SSDECT.SystemMemory;
                    }

                    // Compatible içindeki SystemBattery değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemBattery) && !Compatible.SystemBattery.Contains("{0}"))
                    {
                        return SSDECT.SystemBattery;
                    }

                    // Compatible içindeki SystemGraphic değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemGraphic) && !Compatible.SystemGraphic.Contains("{0}"))
                    {
                        return SSDECT.SystemGraphic;
                    }

                    // Compatible içindeki SystemNetwork değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemNetwork) && !Compatible.SystemNetwork.Contains("{0}"))
                    {
                        return SSDECT.SystemNetwork;
                    }

                    // Compatible içindeki SystemMotherboard değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.SystemMotherboard) && !Compatible.SystemMotherboard.Contains("{0}"))
                    {
                        return SSDECT.SystemMotherboard;
                    }
                }

                return SSDECT.Pass;
            }
            catch
            {
                return SSDECT.UnforeseenConsequences;
            }
        }
    }
}