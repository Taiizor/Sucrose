using System.Windows.Media;
using SSDESHT = Sucrose.Shared.Dependency.Enum.StretchType;

namespace Sucrose.Shared.Dependency.Manage
{
    internal static class Internal
    {
        public static SSDESHT DefaultStretchType = SSDESHT.None;

        public static Stretch DefaultBackgroundStretch = Stretch.UniformToFill;
    }
}