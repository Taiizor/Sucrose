using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSCHA = Sucrose.Shared.Core.Helper.Attribute;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Update
    {
        public static string GetName(SSCEUT Type)
        {
            return SSCHA.GetDisplay(Type).GetName();
        }

        public static string GetDescription(SSCEUT Type)
        {
            return SSCHA.GetDisplay(Type).GetDescription();
        }
    }
}