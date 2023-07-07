using System.IO;
using System.IO.Compression;
using SECT = Skylark.Enum.CompatibilityType;
using SEWT = Skylark.Enum.WallpaperType;
using SHV = Skylark.Helper.Versionly;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Theme.Shared.Helper
{
    internal class Zip
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
                if (!CheckFile(Archive, "SucroseInfo.json"))
                {
                    return SECT.InfoFile;
                }

                // Arşivdeki SucroseInfo.json dosyasını okuma
                ThemeFile Info = ThemeFile.FromJson(ReadFile(Archive, "SucroseInfo.json"));

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
                if ((int)Info.Type >= Enum.GetValues(typeof(SEWT)).Length)
                {
                    return SECT.Type;
                }

                // Info içindeki Type değerine göre dosya veya url kontrolü
                if (Info.Type == SEWT.Web && !CheckFile(Archive, Info.FileName))
                {
                    return SECT.FileName;
                }
                else if (Info.Type == SEWT.Url && !STSHV.IsUrl(Info.FileName))
                {
                    return SECT.FileName;
                }
                else if (Info.Type == SEWT.Gif && !STSHV.IsUrl(Info.FileName) && !CheckFile(Archive, Info.FileName))
                {
                    return SECT.FileName;
                }
                else if (Info.Type == SEWT.Video && !STSHV.IsUrl(Info.FileName) && !CheckFile(Archive, Info.FileName))
                {
                    return SECT.FileName; //FileNameExtension or FileExtension
                }
                else if (Info.Type == SEWT.Application && !CheckFile(Archive, Info.FileName))
                {
                    return SECT.FileName; //FileNameExtension or FileExtension
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