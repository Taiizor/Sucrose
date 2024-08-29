using SPPT = Sucrose.Pipe.PipeT;

namespace Sucrose.Pipe.Manage
{
    public static class Internal
    {
        public static readonly SPPT LauncherManager = new("Sucrose.Launcher.Pipe");

        public static readonly SPPT BackgroundogManager = new("Sucrose.Backgroundog.Pipe");
    }
}