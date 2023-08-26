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

namespace Sucrose.Shared.Theme.Helper
{
    internal static class Zip
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

                // Arşivde SucroseInfo.json dosyası var mı?
                if (!SSZHZ.CheckFile(Archive, SMR.SucroseInfo))
                {
                    return SSDECT.InfoFile;
                }

                // Arşivdeki SucroseInfo.json dosyasını okuma
                SSTHI Info = SSTHI.FromJson(SSZHZ.ReadFile(Archive, SMR.SucroseInfo));

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

                    // Compatible içindeki ComputerDate değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Compatible.ComputerDate) && !Compatible.ComputerDate.Contains("{0}"))
                    {
                        return SSDECT.ComputerDate;
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