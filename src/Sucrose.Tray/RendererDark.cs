using System.Drawing.Drawing2D;

namespace Sucrose.Tray
{
    public class RendererDark : ToolStripProfessionalRenderer
    {
        public RendererDark() : base(new DarkColorTable())
        {
        }

        protected override void OnRenderArrow(ToolStripArrowRenderEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            Rectangle Rectangle = new(e.ArrowRectangle.Location, e.ArrowRectangle.Size);

            Rectangle.Inflate(-2, -6);

            e.Graphics.DrawLines
            (
                Pens.White, new Point[]
                {
                    new Point(Rectangle.Left, Rectangle.Top),
                    new Point(Rectangle.Right, Rectangle.Top + (Rectangle.Height /2)),
                    new Point(Rectangle.Left, Rectangle.Top + Rectangle.Height)
                }
            );
        }

        protected override void OnRenderItemCheck(ToolStripItemImageRenderEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            Rectangle Rectangle = new(e.ImageRectangle.Location, e.ImageRectangle.Size);

            Rectangle.Inflate(-4, -6);

            e.Graphics.DrawLines
            (
                Pens.White, new Point[]
                {
                    new Point(Rectangle.Left, Rectangle.Bottom - (Rectangle.Height /2)),
                    new Point(Rectangle.Left + (Rectangle.Width /3),  Rectangle.Bottom),
                    new Point(Rectangle.Right, Rectangle.Top)
                }
            );
        }

        protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
        {
            e.Item.ForeColor = Color.WhiteSmoke;
            base.OnRenderItemText(e);
        }

        protected override void OnRenderMenuItemBackground(ToolStripItemRenderEventArgs e)
        {
            if (!e.Item.Selected)
            {
                base.OnRenderMenuItemBackground(e);
            }
            else
            {
                SolidBrush FillColor = new(Color.FromArgb(75, 75, 75));
                Pen BorderColor = new(Color.FromArgb(75, 75, 75));

                Rectangle Rectangle = new(Point.Empty, e.Item.Size);

                e.Graphics.FillRectangle(FillColor, Rectangle);
                e.Graphics.DrawRectangle(BorderColor, 1, 0, Rectangle.Width - 2, Rectangle.Height - 1);

                FillColor.Dispose();
                BorderColor.Dispose();
            }
        }
    }
}