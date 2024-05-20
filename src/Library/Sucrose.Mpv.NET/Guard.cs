namespace Sucrose.Mpv.NET
{
    internal static class Guard
    {
        public static void AgainstDisposed(bool disposed, string objectName)
        {
            if (disposed)
            {
                throw new ObjectDisposedException(objectName);
            }
        }

        public static void AgainstNull(object value)
        {
            if (value == null)
            {
                throw new ArgumentNullException();
            }
        }

        public static void AgainstNull(object value, string name)
        {
            if (value == null)
            {
                throw new ArgumentNullException(name);
            }
        }

        public static void AgainstNullOrEmptyOrWhiteSpaceString(string value, string name)
        {
            AgainstNull(value, name);

            if (string.IsNullOrWhiteSpace(value))
            {
                throw new ArgumentException(name);
            }
        }
    }
}