using System.IO;

namespace Sucrose.Shared.SevenZip.Helper
{
    internal static class Zip
    {
        public static bool CheckArchive(string Archive)
        {
            try
            {
                using FileStream Stream = new(Archive, FileMode.Open, FileAccess.Read, FileShare.Read);

                byte[] ArchiveHeader = new byte[7];

                _ = Stream.Read(ArchiveHeader, 0, 7);

                byte[] ZipHeader = new byte[] { 0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00 };

                return ArchiveHeader.SequenceEqual(ZipHeader);
            }
            catch
            {
                return false;
            }
        }
    }
}