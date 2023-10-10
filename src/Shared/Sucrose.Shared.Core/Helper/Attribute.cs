using System.ComponentModel.DataAnnotations;
using System.Reflection;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCEFT = Sucrose.Shared.Core.Enum.FrameworkType;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Attribute
    {
        public static DisplayAttribute GetDisplay(SSCEFT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }

        public static DisplayAttribute GetDisplay(SSCEUT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }

        public static DisplayAttribute GetDisplay(SSCECT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }
    }
}