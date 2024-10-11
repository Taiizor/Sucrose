namespace Sucrose.Memory.Manage.Valuable
{
    public static class Log
    {
        public static string FileNameDate => $"{DateTime.Now:yy.MM.dd}";

        public static string FileTimeLine => $"{DateTime.Now:HH:mm:ss}";

        public static string FileNameCompress => $"sucrose_log_{DateTime.Now:yyyyMMdd_HHmmss}.zip";
    }
}