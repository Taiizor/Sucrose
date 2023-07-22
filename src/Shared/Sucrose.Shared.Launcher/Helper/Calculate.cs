using SWHWT = Skylark.Wing.Helper.WindowsTaskbar;

namespace Sucrose.Shared.Launcher.Helper
{
    internal static class Calculate
    {
        public static Point MenuPosition(Size Size)
        {
            Point MousePosition = Control.MousePosition;

            MousePosition.Offset(-(Size.Width / 2), -(30 + Size.Height));

            return MousePosition;
        }

        public static Point MenuPosition(ContextMenuStrip Menu)
        {
            Point MousePosition = Control.MousePosition;

            AnchorStyles Anchor = SWHWT.GetAnchorStyle();
            Rectangle TaskbarPosition = SWHWT.GetPosition();

            return Anchor switch
            {
                AnchorStyles.Top => new Point(MousePosition.X - (Menu.Width / 2), TaskbarPosition.Bottom + 5),
                AnchorStyles.Bottom => new Point(MousePosition.X - (Menu.Width / 2), TaskbarPosition.Top - Menu.Height - 5),
                AnchorStyles.Left => new Point(TaskbarPosition.Right + 5, MousePosition.Y - (Menu.Height / 2)),
                AnchorStyles.Right => new Point(TaskbarPosition.Left - Menu.Width - 5, MousePosition.Y - (Menu.Height / 2)),
                _ => MousePosition,
            };
        }
    }
}