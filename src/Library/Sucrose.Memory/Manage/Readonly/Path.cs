namespace Sucrose.Memory.Manage.Readonly
{
    public static class Path
    {
        public static readonly string Desktop = Environment.GetFolderPath(Environment.SpecialFolder.Desktop);

        public static readonly string MyDocuments = Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments);

        public static readonly string ApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);

        public static readonly string LocalApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

        public static readonly string CommonApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData);
    }
}