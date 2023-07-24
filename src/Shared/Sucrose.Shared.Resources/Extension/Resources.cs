using Application = System.Windows.Application;

namespace Sucrose.Shared.Resources.Extension
{
    internal static class Resources
    {
        public static string GetValue(string Area, string Key)
        {
            string Resource = Area + "." + Key;

            string Result = Application.Current.TryFindResource(Resource) as string;

            if (string.IsNullOrEmpty(Result))
            {
                return $"[{Resource}]";
            }
            else
            {
                return Result;
            }
        }
    }
}
