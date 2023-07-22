using SSEVMI = Sucrose.Shared.Engine.Vexana.Manage.Internal;

namespace Sucrose.Shared.Engine.Vexana.Helper
{
    internal static class Gif
    {
        public static void Pause()
        {
            SSEVMI.ImageState = false;
        }

        public static void Play()
        {
            SSEVMI.ImageState = true;
        }

        public static void SetLoop(bool State)
        {
            SSEVMI.ImageLoop = State;
        }
    }
}