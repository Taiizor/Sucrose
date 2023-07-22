using System.IO;
using System.IO.Compression;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;

namespace Sucrose.Shared.Zip.Extension
{
    internal static class Zip
    {
        public static SSDECT Extract(string Archive, string Destination)
        {
            try
            {
                // ZIP dosyasını açma
#if NET48_OR_GREATER
                ZipFile.ExtractToDirectory(Archive, Destination);
#else
                ZipFile.ExtractToDirectory(Archive, Destination, true);
#endif

                return SSDECT.Pass;
            }
            catch
            {
                return SSDECT.UnforeseenConsequences;
            }
        }

        public static SSDECT Compress(string Source, string Destination)
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

                return SSDECT.Pass;
            }
            catch
            {
                return SSDECT.UnforeseenConsequences;
            }
        }
    }
}