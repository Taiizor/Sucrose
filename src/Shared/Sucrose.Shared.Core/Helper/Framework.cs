using SSCEFT = Sucrose.Shared.Core.Enum.FrameworkType;
using SSCHA = Sucrose.Shared.Core.Helper.Attribute;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Framework
    {
        public static SSCEFT Get()
        {
#if NET481
            return SSCEFT.NET_Framework_4_8_1;
#elif NET48
            return SSCEFT.NET_Framework_4_8;
#elif NET8_0
            return SSCEFT.NET_8_0;
#elif NET7_0
            return SSCEFT.NET_7_0;
#elif NET6_0
            return SSCEFT.NET_6_0;
#else
            return SSCEFT.Unknown;
#endif
        }

        public static string GetText()
        {
            return $"{Get()}";
        }

        public static string GetName()
        {
            return SSCHA.GetDisplay(Get()).GetName();
        }

        public static string GetDescription()
        {
            return SSCHA.GetDisplay(Get()).GetDescription();
        }
    }
}