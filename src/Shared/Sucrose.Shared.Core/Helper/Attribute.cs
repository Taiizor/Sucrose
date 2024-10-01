using System.ComponentModel.DataAnnotations;
using System.Reflection;
using SSCEFT = Sucrose.Shared.Core.Enum.FrameworkType;
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Attribute
    {
        public static DisplayAttribute GetDisplay(SSCEFT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }

        public static DisplayAttribute GetDisplay(SSCEUCT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }

        public static DisplayAttribute GetDisplay(SSCEUET Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }
    }
}