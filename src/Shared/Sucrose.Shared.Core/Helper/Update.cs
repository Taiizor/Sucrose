using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;
using SSCHA = Sucrose.Shared.Core.Helper.Attribute;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Update
    {
        public static string GetName(SSCEUCT Type)
        {
            return SSCHA.GetDisplay(Type).GetName();
        }

        public static string GetName(SSCEUET Type)
        {
            return SSCHA.GetDisplay(Type).GetName();
        }

        public static string GetDescription(SSCEUCT Type)
        {
            return SSCHA.GetDisplay(Type).GetDescription();
        }

        public static string GetDescription(SSCEUET Type)
        {
            return SSCHA.GetDisplay(Type).GetDescription();
        }
    }
}