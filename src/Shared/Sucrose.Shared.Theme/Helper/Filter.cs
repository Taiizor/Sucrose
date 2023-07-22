using System.IO;
using SEAET = Skylark.Enum.AppExtensionType;
using SEVET = Skylark.Enum.VideoExtensionType;
using SEWET = Skylark.Enum.WebExtensionType;

namespace Sucrose.Shared.Theme.Helper
{
    internal static class Filter
    {
        public static bool AppExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEAET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEAET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }

        public static bool GifExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Extension.Equals("GIF", StringComparison.OrdinalIgnoreCase);
            }
            catch
            {
                return false;
            }
        }

        public static bool WebExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEWET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEWET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }

        public static bool VideoExtension(string File)
        {
            try
            {
                string Extension = Path.GetExtension(File).Replace(".", "");

                return Enum.TryParse<SEVET>(Extension, true, out _);
                //return Enum.IsDefined(typeof(SEVET), Extension.ToUpperInvariant());
            }
            catch
            {
                return false;
            }
        }
    }
}