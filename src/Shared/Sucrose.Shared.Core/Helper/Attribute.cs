using System.ComponentModel.DataAnnotations;
using System.Reflection;
using SSCEBT = Sucrose.Shared.Core.Enum.BundleType;
using SSCEFT = Sucrose.Shared.Core.Enum.FrameworkType;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Attribute
    {
        public static DisplayAttribute GetDisplay(SSCEBT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }

        public static DisplayAttribute GetDisplay(SSCEFT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }
    }
}