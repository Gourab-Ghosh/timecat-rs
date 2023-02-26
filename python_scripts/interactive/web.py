from flask import Flask, url_for

def create_app():
    app = Flask(__name__)
    from .main.routes import main
    app.register_blueprint(main)
    return app