using System.IO;
using System.Windows.Media.Imaging;
using SSEVMI = Sucrose.Shared.Engine.Vexana.Manage.Internal;

namespace Sucrose.Shared.Engine.Vexana.Event
{
    internal static class Gif
    {
        public static async void ImageTimer_Tick(object sender, EventArgs e)
        {
            SSEVMI.ImageTimer.Stop();

            if (SSEVMI.ImageLoop || SSEVMI.ImageFirst)
            {
                SSEVMI.ImageFirst = false;

                foreach (string Image in SSEVMI.ImageResult.List)
                {
                    while (!SSEVMI.ImageState)
                    {
                        await Task.Delay(1000);
                    }

                    SetImage(Image);

                    string[] Split = Path.GetFileNameWithoutExtension(Image).Split('_');

                    //Thread.Sleep(Convert.ToInt32(Split.Last()));
                    await Task.Delay(Convert.ToInt32(Split.Last()));

                    //Thread.Sleep(Convert.ToInt32(Result.Total / Result.List.Count));
                    //await Task.Delay(Convert.ToInt32(Result.Total / Result.List.Count));
                }
            }

            SSEVMI.ImageTimer.Start();
        }

        public static void SetImage(string Path)
        {
            Uri Uri = new(Path);

            BitmapImage Image = new(Uri);

            SSEVMI.ImageEngine.Source = Image;
        }
    }
}