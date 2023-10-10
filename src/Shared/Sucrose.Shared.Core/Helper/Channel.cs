using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCHA = Sucrose.Shared.Core.Helper.Attribute;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Channel
    {
        public static string GetName(SSCECT Type)
        {
            return SSCHA.GetDisplay(Type).GetName();
        }

        public static string GetDescription(SSCECT Type)
        {
            return SSCHA.GetDisplay(Type).GetDescription();
        }
    }
}