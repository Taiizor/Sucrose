using System.IO;
using System.IO.Compression;
using SDECT = Sucrose.Dependency.Enum.CompatibilityType;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;
using SEAET = Skylark.Enum.AppExtensionType;
using SEVET = Skylark.Enum.VideoExtensionType;
using SEWET = Skylark.Enum.WebExtensionType;
using SHV = Skylark.Helper.Versionly;
using SMR = Sucrose.Memory.Readonly;
using STSHI = Sucrose.Theme.Shared.Helper.Info;
using STSHP = Sucrose.Theme.Shared.Helper.Properties;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Theme.Shared.Helper
{
    internal static class Zip
    {
        public static SDECT Extract(string Archive, string Destination)
        {
            try
            {
                // ZIP dosyasını açma
#if NET48_OR_GREATER
                ZipFile.ExtractToDirectory(Archive, Destination);
#else
                ZipFile.ExtractToDirectory(Archive, Destination, true);
#endif

                return SDECT.Pass;
            }
            catch
            {
                return SDECT.UnforeseenConsequences;
            }
        }

        public static SDECT Compress(string Source, string Destination)
        {
            try
            {
                // Eğer ZIP dosyası varsa silme
                if (File.Exists(Destination))
                {
                    File.Delete(Destination);
                }

                // ZIP dosyası oluşturma
#if NET48_OR_GREATER
                ZipFile.CreateFromDirectory(Source, Destination, CompressionLevel.Fastest, false);
#else
                ZipFile.CreateFromDirectory(Source, Destination, CompressionLevel.SmallestSize, false);
#endif

                return SDECT.Pass;
            }
            catch
            {
                return SDECT.UnforeseenConsequences;
            }
        }

        public static SDECT Check(string Archive)
        {
            try
            {
                // Seçilen dosya var mı?
                if (!File.Exists(Archive))
                {
                    return SDECT.NotFound;
                }

                // Seçilen dosya .zip uzantılı değil mi?
                if (Path.GetExtension(Archive) != ".zip")
                {
                    return SDECT.Extension;
                }

                // Seçilen dosya gerçekten ZIP dosyası mı?
                if (!CheckArchive(Archive))
                {
                    return SDECT.ZipType;
                }

                // Arşivde SucroseInfo.json dosyası var mı?
                if (!CheckFile(Archive, SMR.SucroseInfo))
                {
                    return SDECT.InfoFile;
                }

                // Arşivdeki SucroseInfo.json dosyasını okuma
                STSHI Info = STSHI.FromJson(ReadFile(Archive, SMR.SucroseInfo));

                // Info içindeki Thumbnail dosyası var mı?
                if (!CheckFile(Archive, Info.Thumbnail))
                {
                    return SDECT.Thumbnail;
                }

                // Info içindeki Preview dosyası var mı?
                if (!CheckFile(Archive, Info.Preview))
                {
                    return SDECT.Preview;
                }

                // Info içindeki AppVersion sürümü bu uygulamanın düşük mü?
                if (Info.AppVersion.CompareTo(SHV.Executing()) > 0)
                {
                    return SDECT.AppVersion;
                }

                // Info içindeki Type değeri bu uygulamanın Type enum değerinden büyük mü?
                if ((int)Info.Type >= Enum.GetValues(typeof(SDEWT)).Length)
                {
                    return SDECT.Type;
                }

                // Info içindeki Type değerine göre dosya veya url kontrolü
                if (Info.Type == SDEWT.Web)
                {
                    if (!CheckFile(Archive, Info.Source))
                    {
                        return SDECT.Source;
                    }
                    else if (!CheckWebExtension(Info.Source))
                    {
                        return SDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.Url && !STSHV.IsUrl(Info.Source))
                {
                    return SDECT.InvalidUrl;
                }
                else if (Info.Type == SDEWT.Gif)
                {
                    if (!STSHV.IsUrl(Info.Source) && !CheckFile(Archive, Info.Source))
                    {
                        return SDECT.Source;
                    }
                    else if (!CheckGifExtension(Info.Source))
                    {
                        return SDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.Video)
                {
                    if (!STSHV.IsUrl(Info.Source) && !CheckFile(Archive, Info.Source))
                    {
                        return SDECT.Source;
                    }
                    else if (!CheckVideoExtension(Info.Source))
                    {
                        return SDECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.YouTube && !STSHV.IsYouTube(Info.Source) && !STSHV.IsYouTubeMusic(Info.Source))
                {
                    return SDECT.InvalidUrl;
                }
                else if (Info.Type == SDEWT.Application)
                {
                    if (!CheckFile(Archive, Info.Source))
                    {
                        return SDECT.Source;
                    }
                    else if (!CheckAppExtension(Info.Source))
                    {
                        return SDECT.InvalidExtension;
                    }
                }

                // Arşivde SucroseProperties.json dosyası var mı?
                if (CheckFile(Archive, SMR.SucroseProperties))
                {
                    // Arşivdeki SucroseProperties.json dosyasını okuma
                    STSHP Properties = STSHP.FromJson(ReadFile(Archive, SMR.SucroseProperties));

                    // Properties içindeki TriggerTime değeri 1'den küçük mü?
                    if (Properties.TriggerTime <= 0)
                    {
                        return SDECT.TriggerTime;
                    }

                    // Properties içindeki LoopMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.LoopMode) && !Properties.LoopMode.Contains("{0}"))
                    {
                        return SDECT.LoopMode;
                    }

                    // Properties içindeki VolumeLevel değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.VolumeLevel) && !Properties.VolumeLevel.Contains("{0}"))
                    {
                        return SDECT.VolumeLevel;
                    }

                    // Properties içindeki ShuffleMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.ShuffleMode) && !Properties.ShuffleMode.Contains("{0}"))
                    {
                        return SDECT.ShuffleMode;
                    }

                    // Properties içindeki StretchMode değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.StretchMode) && !Properties.StretchMode.Contains("{0}"))
                    {
                        return SDECT.StretchMode;
                    }

                    // Properties içindeki ComputerDate değeri boş değil ve {0} içermiyor mu?
                    if (!string.IsNullOrEmpty(Properties.ComputerDate) && !Properties.ComputerDate.Contains("{0}"))
                    {
                        return SDECT.ComputerDate;
                    }
                }

                return SDECT.Pass;
            }
            catch
            {
                return SDECT.UnforeseenConsequences;
            }
        }

        private static string ReadFile(string Archive, string File)
        {
            try
            {
                using ZipArchive Archives = ZipFile.OpenRead(Archive);

                ZipArchiveEntry Entry = Archives.GetEntry(File);

                using StreamReader Reader = new(Entry.Open());

                return Reader.ReadToEnd();
            }
            catch
            {
                return string.Empty;
            }
        }

        private static bool CheckAppExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEAET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEAET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }

        private static bool CheckGifExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Extension.Equals("GIF", StringComparison.OrdinalIgnoreCase);
            }
            catch
            {
                return false;
            }
        }

        private static bool CheckWebExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEWET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEWET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }

        private static bool CheckVideoExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEVET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEVET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }

        private static bool CheckFile(string Archive, string File, StringComparison Comparison = StringComparison.Ordinal)
        {
            try
            {
                using ZipArchive Archives = ZipFile.OpenRead(Archive);

                return Archives.Entries.Any(Entry => string.Equals(Entry.Name, File, Comparison));
            }
            catch
            {
                return false;
            }
        }

        private static bool CheckArchive(string Archive)
        {
            try
            {
                using FileStream Stream = new(Archive, FileMode.Open, FileAccess.Read, FileShare.Read);

                byte[] ArchiveHeader = new byte[4];
                Stream.Read(ArchiveHeader, 0, 4);

                byte[] ZipHeader = new byte[] { 0x50, 0x4B, 0x03, 0x04 };

                return ArchiveHeader.SequenceEqual(ZipHeader);
            }
            catch
            {
                return false;
            }
        }
    }
}