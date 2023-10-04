using SSCEBT = Sucrose.Shared.Core.Enum.BundleType;
using SSCHA = Sucrose.Shared.Core.Helper.Attribute;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Bundle
    {
        public static string GetName(SSCEBT Type)
        {
            return SSCHA.GetDisplay(Type).GetName();
        }

        public static string GetDescription(SSCEBT Type)
        {
            return SSCHA.GetDisplay(Type).GetDescription();
        }
    }
}