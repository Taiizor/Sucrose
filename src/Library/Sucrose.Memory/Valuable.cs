namespace Sucrose.Memory
{
    public static class Valuable
    {
        public static string LogFileTime => $"{DateTime.Now:HH:mm:ss}";

        public static string LogFileDate => $"{DateTime.Now:yy.MM.dd}";

        public static string LogCompress => $"sucrose_log_{DateTime.Now:yyyyMMdd_HHmmss}.zip";
    }
}