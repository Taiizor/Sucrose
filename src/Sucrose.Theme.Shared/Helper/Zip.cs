using System.IO;
using System.IO.Compression;
using SEAET = Skylark.Enum.AppExtensionType;
using SECT = Skylark.Enum.CompatibilityType;
using SEVET = Skylark.Enum.VideoExtensionType;
using SEWET = Skylark.Enum.WebExtensionType;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;
using SHV = Skylark.Helper.Versionly;
using SMR = Sucrose.Memory.Readonly;
using STSHI = Sucrose.Theme.Shared.Helper.Info;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Theme.Shared.Helper
{
    internal static class Zip
    {
        public static SECT Extract(string Archive, string Destination)
        {
            try
            {
                // ZIP dosyasını açma
#if NET48_OR_GREATER
                ZipFile.ExtractToDirectory(Archive, Destination);
#else
                ZipFile.ExtractToDirectory(Archive, Destination, true);
#endif

                return SECT.Pass;
            }
            catch
            {
                return SECT.UnforeseenConsequences;
            }
        }

        public static SECT Compress(string Source, string Destination)
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

                return SECT.Pass;
            }
            catch
            {
                return SECT.UnforeseenConsequences;
            }
        }

        public static SECT Check(string Archive)
        {
            try
            {
                // Seçilen dosya var mı?
                if (!File.Exists(Archive))
                {
                    return SECT.NotFound;
                }

                // Seçilen dosya .zip uzantılı değil mi?
                if (Path.GetExtension(Archive) != ".zip")
                {
                    return SECT.Extension;
                }

                // Seçilen dosya gerçekten ZIP dosyası mı?
                if (!CheckArchive(Archive))
                {
                    return SECT.ZipType;
                }

                // Arşivde SucroseInfo.json dosyası var mı?
                if (!CheckFile(Archive, SMR.SucroseInfo))
                {
                    return SECT.InfoFile;
                }

                // Arşivdeki SucroseInfo.json dosyasını okuma
                STSHI Info = STSHI.FromJson(ReadFile(Archive, SMR.SucroseInfo));

                // Info içindeki Thumbnail dosyası var mı?
                if (!CheckFile(Archive, Info.Thumbnail))
                {
                    return SECT.Thumbnail;
                }

                // Info içindeki Preview dosyası var mı?
                if (!CheckFile(Archive, Info.Preview))
                {
                    return SECT.Preview;
                }

                // Info içindeki AppVersion sürümü bu uygulamanın düşük mü?
                if (Info.AppVersion.CompareTo(SHV.Executing()) > 0)
                {
                    return SECT.AppVersion;
                }

                // Info içindeki Type değeri bu uygulamanın Type enum değerinden büyük mü?
                if ((int)Info.Type >= Enum.GetValues(typeof(SDEWT)).Length)
                {
                    return SECT.Type;
                }

                // Info içindeki Type değerine göre dosya veya url kontrolü
                if (Info.Type == SDEWT.Web)
                {
                    if (!CheckFile(Archive, Info.FileName))
                    {
                        return SECT.FileName;
                    }
                    else if (!CheckWebExtension(Info.FileName))
                    {
                        return SECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.Url && !STSHV.IsUrl(Info.FileName))
                {
                    return SECT.InvalidUrl;
                }
                else if (Info.Type == SDEWT.Gif)
                {
                    if (!STSHV.IsUrl(Info.FileName) && !CheckFile(Archive, Info.FileName))
                    {
                        return SECT.FileName;
                    }
                    else if (!CheckGifExtension(Info.FileName))
                    {
                        return SECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.Video)
                {
                    if (!STSHV.IsUrl(Info.FileName) && !CheckFile(Archive, Info.FileName))
                    {
                        return SECT.FileName;
                    }
                    else if (!CheckVideoExtension(Info.FileName))
                    {
                        return SECT.InvalidExtension;
                    }
                }
                else if (Info.Type == SDEWT.YouTube && !STSHV.IsYouTube(Info.FileName) && !STSHV.IsYouTubeMusic(Info.FileName))
                {
                    return SECT.InvalidUrl;
                }
                else if (Info.Type == SDEWT.Application)
                {
                    if (!CheckFile(Archive, Info.FileName))
                    {
                        return SECT.FileName;
                    }
                    else if (!CheckAppExtension(Info.FileName))
                    {
                        return SECT.InvalidExtension;
                    }
                }

                return SECT.Pass;
            }
            catch
            {
                return SECT.UnforeseenConsequences;
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