using System.ComponentModel.DataAnnotations;
using System.Reflection;
using SSCEFT = Sucrose.Shared.Core.Enum.FrameworkType;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Attribute
    {
        public static DisplayAttribute GetDisplay(SSCEFT Enum)
        {
            FieldInfo Field = Enum.GetType().GetField(Enum.ToString());

            return Field?.GetCustomAttribute<DisplayAttribute>();
        }
    }
}