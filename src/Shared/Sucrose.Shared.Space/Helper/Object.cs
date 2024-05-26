using System.Management;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Object
    {
        public static string Check(ManagementObject Object, string Title, string Back)
        {
            try
            {
                string Value = $"{Object[Title]}";

                if (string.IsNullOrEmpty(Value) || string.IsNullOrWhiteSpace(Value))
                {
                    return Back;
                }
                else
                {
                    return Value.TrimStart().TrimEnd();
                }
            }
            catch
            {
                return Back;
            }
        }
    }
}