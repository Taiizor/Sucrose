using SEVAMI = Sucrose.Engine.VA.Manage.Internal;

namespace Sucrose.Engine.VA.Helper
{
    internal static class Gif
    {
        public static void Pause()
        {
            SEVAMI.ImageState = false;
        }

        public static void Play()
        {
            SEVAMI.ImageState = true;
        }

        public static void SetLoop(bool State)
        {
            SEVAMI.ImageLoop = State;
        }
    }
}