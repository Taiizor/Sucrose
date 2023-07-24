using System.IO;
using System.IO.Compression;

namespace Sucrose.Shared.Zip.Helper
{
    internal static class Zip
    {
        public static string EntryName(string File, string Source)
        {
            try
            {
                string Relative = File.Substring(Source.Length);

                Relative = Relative.TrimStart(Path.DirectorySeparatorChar);

                return Path.Combine(Path.GetFileName(Path.GetDirectoryName(File)), Relative);
            }
            catch
            {
                return string.Empty;
            }
        }

        public static string ReadFile(string Archive, string File)
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

        public static bool CheckFile(string Archive, string File, StringComparison Comparison = StringComparison.Ordinal)
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

        public static bool CheckArchive(string Archive)
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